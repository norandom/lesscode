//! Read-only source inventories under explicit, frozen path rules.

use crate::model::{
    Exclusion, ExclusionKind, FileRecord, InventoryReference, Manifest, Records, SchemaVersion,
    Selector,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub files: Records<String, FileRecord>,
    pub unclassified: Records<String, String>,
    pub excluded_files: Records<String, String>,
    pub exclusions: Records<String, Exclusion>,
    pub identity: String,
}

/// Invalid paths and unreadable inputs are fatal, with their path in the message.
#[derive(Debug)]
pub struct ScopeError(pub String);

impl std::fmt::Display for ScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ScopeError {}

/// Inventory every regular file, without consulting Git or ignore files.
/// Rule keys are normalized relative exact paths or directory prefixes; `.`
/// matches the root. The longest category prefix wins. Generated/tool-state
/// exclusions prune traversal; excluded maintained source retains its digest.
/// No exclusions are implicit, including `.git`. Categories may be absent;
/// those file digests are retained for later changed-file admission.
/// This checks a supplied inventory's source binding; call
/// `verify_capability_inventory` as well to validate its external artifact.
pub fn inventory(root: &Path, manifest: &Manifest) -> Result<Snapshot, ScopeError> {
    let mut spellings = BTreeMap::new();
    for path in manifest
        .categories
        .0
        .keys()
        .chain(manifest.exclusions.0.keys())
    {
        validate_path(path, true)?;
        check_spelling(&mut spellings, path)?;
    }
    for path in &manifest.wider_boundary {
        validate_path(path, true)?;
    }
    for selector in manifest
        .owned
        .0
        .values()
        .chain(manifest.entry_points.0.values())
    {
        let (Selector::WholeFile { path } | Selector::Symbol { path, .. }) = selector;
        validate_path(path, false)?;
        if manifest
            .exclusions
            .0
            .keys()
            .any(|rule| matches_rule(path, rule))
        {
            return Err(ScopeError(format!(
                "declared owned/entry source is excluded: {path}"
            )));
        }
    }
    reject_link(root)?;
    if !root.is_dir() {
        return Err(ScopeError(format!(
            "not a source directory: {}",
            root.display()
        )));
    }
    let mut contents = BTreeMap::new();
    collect(root, "", manifest, &mut spellings, &mut contents)?;
    let mut snapshot = Snapshot {
        files: Records(BTreeMap::new()),
        unclassified: Records(BTreeMap::new()),
        excluded_files: Records(BTreeMap::new()),
        exclusions: manifest.exclusions.clone(),
        identity: String::new(),
    };
    let mut hash = Sha256::new();
    field(&mut hash, b"lesscode.source.v1");
    field(
        &mut hash,
        &serde_json::to_vec(&manifest.categories).map_err(|e| ScopeError(e.to_string()))?,
    );
    field(
        &mut hash,
        &serde_json::to_vec(&manifest.exclusions).map_err(|e| ScopeError(e.to_string()))?,
    );
    for (path, bytes) in contents {
        field(&mut hash, path.as_bytes());
        field(&mut hash, &bytes);
        let content_digest = digest(&bytes);
        if rule(&path, &manifest.exclusions.0).is_some() {
            snapshot.excluded_files.0.insert(path, content_digest);
        } else if let Some(category) = rule(&path, &manifest.categories.0) {
            snapshot.files.0.insert(
                path,
                FileRecord {
                    schema_version: SchemaVersion::V1,
                    content_digest,
                    category: *category,
                },
            );
        } else {
            snapshot.unclassified.0.insert(path, content_digest);
        }
    }
    snapshot.identity = hex(&hash.finalize());
    if let Some(reference) = &manifest.capability_inventory {
        verify_binding(&snapshot, reference)?;
    }
    Ok(snapshot)
}

fn field(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}
fn digest(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn matches_rule(path: &str, rule: &str) -> bool {
    rule == "."
        || path == rule
        || path
            .strip_prefix(rule)
            .is_some_and(|tail| tail.starts_with('/'))
}
fn rule<'a, T>(path: &str, rules: &'a BTreeMap<String, T>) -> Option<&'a T> {
    rules
        .iter()
        .filter(|(key, _)| matches_rule(path, key))
        .max_by_key(|(key, _)| key.len())
        .map(|(_, value)| value)
}
fn validate_path(path: &str, allow_root: bool) -> Result<(), ScopeError> {
    if (allow_root && path == ".")
        || (!path.is_empty()
            && !path.contains(['\\', ':', '\0'])
            && path.split('/').all(|part| {
                !part.is_empty() && part != "." && part != ".." && !part.ends_with(['.', ' '])
            }))
    {
        Ok(())
    } else {
        Err(ScopeError(format!(
            "invalid normalized relative path: {path}"
        )))
    }
}
fn check_spelling(spellings: &mut BTreeMap<String, String>, path: &str) -> Result<(), ScopeError> {
    // Prefixes also collide: `Src/a.py` and `src/b.py` are not portable.
    let mut prefix = String::new();
    for part in path.split('/') {
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(part);
        if let Some(prior) = spellings.insert(prefix.to_lowercase(), prefix.clone()) {
            if prior != prefix {
                return Err(ScopeError(format!(
                    "case-colliding paths: {prior}, {prefix}"
                )));
            }
        }
    }
    Ok(())
}
fn reject_link(path: &Path) -> Result<fs::Metadata, ScopeError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|e| ScopeError(format!("{}: {e}", path.display())))?;
    #[cfg(windows)]
    let reparse = {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    };
    #[cfg(not(windows))]
    let reparse = false;
    if metadata.file_type().is_symlink() || reparse {
        return Err(ScopeError(format!(
            "symlink/reparse point is forbidden: {}",
            path.display()
        )));
    }
    Ok(metadata)
}
fn collect(
    root: &Path,
    relative: &str,
    manifest: &Manifest,
    spellings: &mut BTreeMap<String, String>,
    contents: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), ScopeError> {
    let path = if relative.is_empty() {
        root.to_path_buf()
    } else {
        root.join(relative)
    };
    let metadata = reject_link(&path)?;
    if !relative.is_empty() {
        validate_path(relative, false)?;
        check_spelling(spellings, relative)?;
    }
    let key = if relative.is_empty() { "." } else { relative };
    // Any generated/tool-state ancestor prunes, even if a nested rule exists.
    if manifest.exclusions.0.iter().any(|(rule, exclusion)| {
        matches_rule(key, rule) && exclusion.kind != ExclusionKind::MaintainedSource
    }) {
        return Ok(());
    }
    if metadata.is_dir() {
        let mut entries = fs::read_dir(&path)
            .map_err(|e| ScopeError(format!("{}: {e}", path.display())))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ScopeError(format!("{}: {e}", path.display())))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| ScopeError(format!("non-UTF-8 path in {}", path.display())))?;
            let child = if relative.is_empty() {
                name
            } else {
                format!("{relative}/{name}")
            };
            collect(root, &child, manifest, spellings, contents)?;
        }
    } else if metadata.is_file() {
        let bytes = fs::read(&path).map_err(|e| ScopeError(format!("{}: {e}", path.display())))?;
        contents.insert(relative.into(), bytes);
    } else {
        return Err(ScopeError(format!(
            "not a regular source file: {}",
            path.display()
        )));
    }
    Ok(())
}
fn verify_binding(snapshot: &Snapshot, reference: &InventoryReference) -> Result<(), ScopeError> {
    if reference.source_identity != snapshot.identity {
        return Err(ScopeError(
            "capability inventory source identity mismatch".into(),
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for path in &reference.manifest_references {
        validate_path(path, false)?;
        if !seen.insert(path) {
            return Err(ScopeError(format!(
                "duplicate capability inventory source reference: {path}"
            )));
        }
        if !snapshot.files.0.contains_key(path) && !snapshot.unclassified.0.contains_key(path) {
            return Err(ScopeError(format!(
                "capability inventory source reference is missing or excluded: {path}"
            )));
        }
    }
    Ok(())
}
/// Validate the supplied document's recorded snapshot, source-reference inclusion
/// and raw SHA-256. The caller resolves `reference.artifact.path` to the explicit
/// external `artifact_path`; it is never read relative to a source root.
pub fn verify_capability_inventory(
    snapshot: &Snapshot,
    reference: &InventoryReference,
    artifact_path: &Path,
) -> Result<(), ScopeError> {
    verify_binding(snapshot, reference)?;
    let metadata = reject_link(artifact_path)?;
    if !metadata.is_file() {
        return Err(ScopeError(format!(
            "not an inventory artifact file: {}",
            artifact_path.display()
        )));
    }
    let bytes = fs::read(artifact_path)
        .map_err(|e| ScopeError(format!("{}: {e}", artifact_path.display())))?;
    if digest(&bytes) != reference.artifact.sha256 {
        return Err(ScopeError(format!(
            "capability inventory artifact digest mismatch: {}",
            artifact_path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Artifact, Category, ExclusionKind, SchemaVersion};
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "lesscode-scope-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn write(&self, path: &str, content: &str) {
            let path = self.0.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn manifest() -> Manifest {
        serde_json::from_value(json!({"schema_version":1,"trial_id":"trial","hypothesis":"reuse",
            "entry_points":{"entry":{"kind":"whole_file","path":"src/a.py"}},
            "owned":{"a":{"kind":"whole_file","path":"src/a.py"}},"wider_boundary":["."],
            "categories":{"src":"production","tests":"tests","fixtures":"fixtures",
                "research":"research_support","settings.json":"configuration","Cargo.toml":"dependency"},
            "exclusions":{".git":{"schema_version":1,"rationale":"git state","kind":"tool_state"},
                "build":{"schema_version":1,"rationale":"generated output","kind":"generated"}},
            "scope_review":{"schema_version":1,"rationale":"owned responsibility","relationships":{}},
            "checks":{},"target":null,"capability_inventory":null})).unwrap()
    }
    #[test]
    fn relocated_roots_include_untracked_and_preserve_source() {
        let a = Fixture::new();
        let b = Fixture::new();
        for root in [&a, &b] {
            root.write("src/a.py", "return_value = 1\n");
            root.write("src/new.py", "helper = 2\n");
            root.write(".gitignore", "src/new.py\n");
            root.write(".git/config", "different git metadata");
            root.write("build/output", "generated");
        }
        let frozen = inventory(&a.0, &manifest()).unwrap();
        b.write("build/output", "another generated file");
        assert_eq!(frozen, inventory(&b.0, &manifest()).unwrap());
        assert_eq!(frozen.files.0["src/new.py"].category, Category::Production);
        assert!(frozen.unclassified.0.contains_key(".gitignore"));
        assert_eq!(frozen.exclusions.0["build"].kind, ExclusionKind::Generated);
        assert_eq!(
            fs::read_to_string(a.0.join("src/a.py")).unwrap(),
            "return_value = 1\n"
        );
        assert_eq!(frozen, inventory(&a.0, &manifest()).unwrap());
        b.write("src/a.py", "return_value = 2\n");
        assert_ne!(
            frozen.identity,
            inventory(&b.0, &manifest()).unwrap().identity
        );
    }
    #[test]
    fn explicit_rules_keep_all_categories_and_have_no_implicit_exclusions() {
        let root = Fixture::new();
        for path in [
            "src/a.py",
            "tests/test_a.py",
            "fixtures/case.txt",
            "research/run.txt",
            "settings.json",
            "Cargo.toml",
        ] {
            root.write(path, "{}");
        }
        let mut rules = manifest();
        rules
            .categories
            .0
            .insert("src/special".into(), Category::Fixtures);
        root.write("src/special/data.py", "{}");
        let snapshot = inventory(&root.0, &rules).unwrap();
        for (path, category) in [
            ("src/a.py", Category::Production),
            ("tests/test_a.py", Category::Tests),
            ("fixtures/case.txt", Category::Fixtures),
            ("research/run.txt", Category::ResearchSupport),
            ("settings.json", Category::Configuration),
            ("Cargo.toml", Category::Dependency),
            ("src/special/data.py", Category::Fixtures),
        ] {
            assert_eq!(snapshot.files.0[path].category, category);
            assert_eq!(
                snapshot.files.0[path].content_digest,
                "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
            );
        }
        rules.exclusions.0.clear();
        root.write(".git/config", "maintained without explicit exclusion");
        assert!(inventory(&root.0, &rules)
            .unwrap()
            .unclassified
            .0
            .contains_key(".git/config"));
    }
    #[test]
    fn categories_exclusions_and_record_boundaries_affect_identity() {
        let root = Fixture::new();
        root.write("src/a.py", "bc");
        root.write("Cargo.toml", "manifest");
        let base = inventory(&root.0, &manifest()).unwrap();
        let mut changed = manifest();
        changed.categories.0.insert("src".into(), Category::Tests);
        assert_ne!(
            base.identity,
            inventory(&root.0, &changed).unwrap().identity
        );
        changed = manifest();
        changed.exclusions.0.insert(
            "src/a.py".into(),
            Exclusion {
                schema_version: SchemaVersion::V1,
                rationale: "excluded source".into(),
                kind: ExclusionKind::MaintainedSource,
            },
        );
        assert!(inventory(&root.0, &changed).is_err()); // declared owned source cannot be excluded
        changed.owned.0.clear();
        changed.entry_points.0.clear();
        let excluded = inventory(&root.0, &changed).unwrap();
        assert!(excluded.excluded_files.0.contains_key("src/a.py"));
        root.write("src/a.py", "changed");
        assert_ne!(
            excluded.identity,
            inventory(&root.0, &changed).unwrap().identity
        );
        let other = Fixture::new();
        other.write("src/ab.py", "c");
        other.write("Cargo.toml", "manifest");
        assert_ne!(
            base.identity,
            inventory(&other.0, &manifest()).unwrap().identity
        );
    }
    #[test]
    fn rejects_escaping_nonportable_and_colliding_rules() {
        let root = Fixture::new();
        root.write("src/a.py", "a");
        for path in [
            "../outside",
            "/absolute",
            "C:/absolute",
            "src\\a.py",
            "src/../a.py",
            "src//a.py",
            "src/./a.py",
        ] {
            let mut rules = manifest();
            rules.categories.0.insert(path.into(), Category::Tests);
            assert!(inventory(&root.0, &rules).is_err(), "{path}");
        }
        let mut rules = manifest();
        rules.categories.0.insert("SRC".into(), Category::Tests);
        assert!(inventory(&root.0, &rules).is_err());
    }
    #[test]
    fn capability_inventory_requires_snapshot_artifact_and_source_binding() {
        let root = Fixture::new();
        root.write("src/a.py", "a");
        root.write("Cargo.toml", "manifest");
        let snap = inventory(&root.0, &manifest()).unwrap();
        let artifact = Fixture::new();
        artifact.write("inventory.json", "{}");
        let path = artifact.0.join("inventory.json");
        let mut reference = InventoryReference {
            schema_version: SchemaVersion::V1,
            source_identity: snap.identity.clone(),
            manifest_references: vec!["Cargo.toml".into()],
            artifact: Artifact {
                schema_version: SchemaVersion::V1,
                path: "inventory.json".into(),
                sha256: "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a".into(),
            },
        };
        verify_capability_inventory(&snap, &reference, &path).unwrap();
        let mut supplied = manifest();
        supplied.capability_inventory = Some(reference.clone());
        assert_eq!(
            inventory(&root.0, &supplied).unwrap().identity,
            snap.identity
        );
        reference.manifest_references = vec!["src/a.py".into()];
        verify_capability_inventory(&snap, &reference, &path).unwrap();
        root.write("src/a.py", "new snapshot");
        assert!(inventory(&root.0, &supplied).is_err());
        reference.source_identity = "stale".into();
        assert!(verify_capability_inventory(&snap, &reference, &path).is_err());
        reference.source_identity = snap.identity.clone();
        reference.manifest_references = vec!["missing.py".into()];
        assert!(verify_capability_inventory(&snap, &reference, &path).is_err());
        reference.manifest_references = vec!["Cargo.toml".into()];
        artifact.write("inventory.json", "changed");
        assert!(verify_capability_inventory(&snap, &reference, &path).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn rejects_windows_junction_even_at_exclusion_boundary() {
        let root = Fixture::new();
        root.write("src/a.py", "a");
        let target = Fixture::new();
        target.write("external.py", "external");
        let link = root.0.join("build");
        let result = std::process::Command::new("cmd.exe")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(&target.0)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let result = inventory(&root.0, &manifest());
        fs::remove_dir(&link).unwrap();
        assert!(result.unwrap_err().0.contains("symlink/reparse"));
        assert_eq!(
            fs::read_to_string(target.0.join("external.py")).unwrap(),
            "external"
        );
    }
    #[cfg(unix)]
    #[test]
    fn rejects_case_collisions_and_symlinks() {
        let root = Fixture::new();
        root.write("src/a.py", "a");
        root.write("src/A.py", "b");
        assert!(inventory(&root.0, &manifest()).is_err());
        fs::remove_file(root.0.join("src/A.py")).unwrap();
        std::os::unix::fs::symlink("a.py", root.0.join("src/link.py")).unwrap();
        assert!(inventory(&root.0, &manifest()).is_err());
    }
}
