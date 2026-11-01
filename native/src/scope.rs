//! Read-only source inventories under explicit, frozen path rules.

use crate::model::{
    Exclusion, ExclusionKind, FileRecord, InventoryReference, Manifest, Records, SchemaVersion,
    Selector,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Half-open byte range, always a complete selected syntax node or file.
/// Symbol offsets lie on UTF-8 boundaries; whole files may contain arbitrary bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub path: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedScope {
    pub entry_points: Records<String, SourceSpan>,
    pub owned: Records<String, SourceSpan>,
    pub spans: Vec<SourceSpan>,
    pub review: crate::model::ScopeReview,
    /// All located unresolved relationships survive review; none imply dead code.
    pub unresolved: Records<String, crate::model::Finding>,
    pub findings: Vec<crate::model::Finding>,
}

/// Resolve the manifest's declared scope against a previously inventoried root.
/// Missing, excluded, stale or unresolvable declared source fails admission,
/// rather than contributing a zero measurement. Incomplete semantic review is
/// retained as findings. Relationship IDs and owned target IDs are the keys of
/// the supplied unresolved map and manifest ownership map respectively.
pub fn resolve_scope(
    root: &Path,
    snapshot: &Snapshot,
    manifest: &Manifest,
    unresolved: &Records<String, crate::model::Finding>,
) -> Result<ResolvedScope, ScopeError> {
    let entry_points = resolve_selections(root, snapshot, &manifest.entry_points)?;
    let owned = resolve_selections(root, snapshot, &manifest.owned)?;
    let spans = union_spans(owned.0.values().cloned().collect());
    let mut findings = Vec::new();
    let mut finding = |subject: String, explanation: &str| {
        findings.push(crate::model::Finding {
            schema_version: SchemaVersion::V1,
            subject,
            explanation: explanation.into(),
        });
    };
    if manifest.scope_review.rationale.trim().is_empty() {
        finding("scope_review".into(), "ownership rationale is missing");
    }
    for (id, entry) in &entry_points.0 {
        if !spans.iter().any(|span| {
            span.path == entry.path && span.start <= entry.start && span.end >= entry.end
        }) {
            finding(
                id.clone(),
                "entry point is omitted or narrowed by declared ownership",
            );
        }
    }
    for id in unresolved.0.keys() {
        if !manifest.scope_review.relationships.0.contains_key(id) {
            finding(
                id.clone(),
                "unresolved relationship has no ownership review",
            );
        }
    }
    for (id, review) in &manifest.scope_review.relationships.0 {
        let external = review
            .external_contract
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty());
        if review.rationale.trim().is_empty() || (review.owned_targets.is_empty() && !external) {
            finding(
                id.clone(),
                "relationship requires rationale and owned targets or an external contract",
            );
        }
        for target in &review.owned_targets {
            if !owned.0.contains_key(target) {
                finding(
                    id.clone(),
                    &format!("reviewed owned target is omitted: {target}"),
                );
            }
        }
    }
    Ok(ResolvedScope {
        entry_points,
        owned,
        spans,
        review: manifest.scope_review.clone(),
        unresolved: unresolved.clone(),
        findings,
    })
}

/// Resolve complete nodes independently for use by later replacement accounting.
/// Symbol digests are SHA-256 of the entire source file, matching FileRecord.
/// Qualified names join enclosing named declarations with `.`. Occurrence is
/// zero-based among exact qualified-name matches in source order, so repeated
/// definitions are explicit rather than silently choosing the first one.
/// Python decorated definitions and TS export declarations include their wrapper.
/// Named variable declarators include the containing declaration (and any sibling
/// declarators), preserving declaration headers and exported wrappers in scope.
pub fn resolve_selections(
    root: &Path,
    snapshot: &Snapshot,
    selections: &Records<String, Selector>,
) -> Result<Records<String, SourceSpan>, ScopeError> {
    let mut resolved = BTreeMap::new();
    for (id, selector) in &selections.0 {
        let (Selector::WholeFile { path } | Selector::Symbol { path, .. }) = selector;
        validate_path(path, false)?;
        let expected = snapshot
            .files
            .0
            .get(path)
            .map(|file| &file.content_digest)
            .or_else(|| snapshot.unclassified.0.get(path))
            .ok_or_else(|| ScopeError(format!("declared source is missing or excluded: {path}")))?;
        // Recheck every component, including the root, before accessing source.
        reject_link(root)?;
        let mut physical = root.to_path_buf();
        for component in path.split('/') {
            physical.push(component);
            reject_link(&physical)?;
        }
        let bytes = fs::read(&physical).map_err(|e| ScopeError(format!("{path}: {e}")))?;
        if digest(&bytes) != *expected {
            return Err(ScopeError(format!(
                "source changed since inventory: {path}"
            )));
        }
        let (start, end) = match selector {
            Selector::WholeFile { .. } => (0, bytes.len()),
            Selector::Symbol {
                qualified_name,
                occurrence,
                source_digest,
                ..
            } => {
                if source_digest != expected {
                    return Err(ScopeError(format!(
                        "selector source digest mismatch: {id} ({path})"
                    )));
                }
                let source = std::str::from_utf8(&bytes)
                    .map_err(|_| ScopeError(format!("symbol source is not UTF-8: {path}")))?;
                let matches = match Path::new(path).extension().and_then(|ext| ext.to_str()) {
                    Some("py" | "pyi") => named_spans(source, crate::Python, qualified_name)?,
                    Some("ts" | "mts" | "cts") => {
                        named_spans(source, crate::TypeScript, qualified_name)?
                    }
                    Some("tsx") => named_spans(source, crate::Tsx, qualified_name)?,
                    _ => {
                        return Err(ScopeError(format!(
                            "unsupported symbol source language: {path}"
                        )))
                    }
                };
                let index = usize::try_from(occurrence.0)
                    .map_err(|_| ScopeError(format!("selector occurrence out of range: {id}")))?;
                let span = matches.get(index).ok_or_else(|| ScopeError(format!("selector does not resolve exactly once: {id} ({qualified_name}, occurrence {})", occurrence.0)))?;
                (span.start, span.end)
            }
        };
        resolved.insert(
            id.clone(),
            SourceSpan {
                path: path.clone(),
                start,
                end,
            },
        );
    }
    Ok(Records(resolved))
}

fn named_spans<L: ast_grep_language::LanguageExt>(
    source: &str,
    language: L,
    name: &str,
) -> Result<Vec<std::ops::Range<usize>>, ScopeError> {
    let tree = crate::AstGrep::new(source, language);
    let root = tree.root();
    if root.get_inner_node().has_error() || root.dfs().any(|node| node.is_missing()) {
        return Err(ScopeError(
            "symbol source contains syntax errors or missing nodes".into(),
        ));
    }
    let named_kind = |kind: &str| {
        matches!(
            kind,
            "function_definition"
                | "class_definition"
                | "function_declaration"
                | "generator_function_declaration"
                | "class_declaration"
                | "abstract_class_declaration"
                | "method_definition"
                | "method_signature"
                | "abstract_method_signature"
                | "function_signature"
                | "variable_declarator"
                | "interface_declaration"
                | "type_alias_declaration"
                | "enum_declaration"
                | "internal_module"
        )
    };
    let mut matches = Vec::new();
    for node in root.dfs() {
        if !named_kind(&node.kind()) {
            continue;
        }
        let Some(identifier) = node.field("name") else {
            continue;
        };
        let mut names = vec![identifier.text().into_owned()];
        let mut parent = node.parent();
        while let Some(ancestor) = parent {
            if named_kind(&ancestor.kind()) {
                if let Some(identifier) = ancestor.field("name") {
                    names.push(identifier.text().into_owned());
                }
            }
            parent = ancestor.parent();
        }
        names.reverse();
        if names.join(".") == name {
            let mut selected = node;
            while let Some(parent) = selected.parent() {
                if matches!(
                    parent.kind().as_ref(),
                    "decorated_definition"
                        | "export_statement"
                        | "lexical_declaration"
                        | "variable_declaration"
                ) {
                    selected = parent;
                } else {
                    break;
                }
            }
            matches.push(selected.range());
        }
    }
    matches.sort_by_key(|range| (range.start, range.end));
    if matches.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ScopeError(format!(
            "ambiguous complete-node resolution: {name}"
        )));
    }
    Ok(matches)
}

/// Deterministic union of overlapping or adjacent spans, separately per file.
pub fn union_spans(mut spans: Vec<SourceSpan>) -> Vec<SourceSpan> {
    spans.sort_by(|a, b| (&a.path, a.start, a.end).cmp(&(&b.path, b.start, b.end)));
    let mut merged: Vec<SourceSpan> = Vec::new();
    for span in spans {
        if let Some(last) = merged.last_mut() {
            if last.path == span.path && span.start <= last.end {
                last.end = last.end.max(span.end);
                continue;
            }
        }
        merged.push(span);
    }
    merged
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub files: Records<String, FileRecord>,
    pub unclassified: Records<String, String>,
    pub excluded_files: Records<String, String>,
    pub exclusions: Records<String, Exclusion>,
    pub identity: String,
}

/// A source root paired with its frozen inventory.
pub struct SourceSide<'a> {
    pub root: &'a Path,
    pub snapshot: &'a Snapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SideAccounting {
    pub selected: Vec<SourceSpan>,
    pub capability: Vec<SourceSpan>,
    pub wider: Vec<SourceSpan>,
    /// Frozen categories for the reported spans; downstream measurements group
    /// these separately rather than treating tests/configuration as production.
    pub categories: BTreeMap<String, crate::model::Category>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplacementAccounting {
    pub baseline: SideAccounting,
    pub candidate: SideAccounting,
    pub changed_paths: Vec<String>,
    pub outside_boundary: Vec<String>,
    pub dependency_paths: Vec<String>,
    pub dependency_changes: BTreeMap<String, crate::model::DependencyChange>,
    pub findings: Vec<crate::model::Finding>,
}

/// Build paired byte-span accounting without measuring or deciding compression.
/// Both snapshots must have been inventoried using the frozen manifest. Findings
/// retain admission failures for the comparison layer; they never erase source.
/// Located unresolved calls are supplied to `resolve_scope` by later integration.
pub fn account_replacement(
    baseline: SourceSide<'_>,
    candidate: SourceSide<'_>,
    manifest: &Manifest,
    evidence: &crate::model::EvidenceBundle,
) -> Result<ReplacementAccounting, ScopeError> {
    let original = resolve_scope(
        baseline.root,
        baseline.snapshot,
        manifest,
        &Records(BTreeMap::new()),
    )?;
    let mut result = ReplacementAccounting {
        findings: original.findings,
        dependency_changes: evidence.dependency_changes.0.clone(),
        ..ReplacementAccounting::default()
    };
    let mut finding = |subject: &str, explanation: &str| {
        result.findings.push(crate::model::Finding {
            schema_version: SchemaVersion::V1,
            subject: subject.into(),
            explanation: explanation.into(),
        });
    };
    for side in [&baseline, &candidate] {
        if side.snapshot.exclusions != manifest.exclusions {
            finding(
                "exclusions",
                "classification changed from frozen exclusions",
            );
        }
        for (path, file) in &side.snapshot.files.0 {
            if rule(path, &manifest.categories.0) != Some(&file.category) {
                finding(path, "classification differs from frozen category rules");
            }
        }
    }
    let ownership = &evidence.candidate_ownership;
    if ownership.review.rationale.trim().is_empty() {
        finding(
            "candidate_ownership",
            "candidate ownership review rationale is missing",
        );
    }
    let mut selected = Vec::new();
    for (id, selectors) in &ownership.replacements.0 {
        if !manifest.owned.0.contains_key(id) {
            finding(
                id,
                "candidate ownership names an unknown baseline owned selector",
            );
        }
        let selections = Records(
            selectors
                .iter()
                .enumerate()
                .map(|(i, selector)| (i.to_string(), selector.clone()))
                .collect(),
        );
        let spans = resolve_selections(candidate.root, candidate.snapshot, &selections)?;
        if selectors.is_empty() {
            finding(id, "candidate owned implementation is omitted");
        }
        selected.extend(spans.0.into_values());
    }
    let candidate_selected = union_spans(selected);
    for (id, span) in &original.owned.0 {
        if !ownership.replacements.0.contains_key(id) {
            finding(id, "candidate owned implementation is omitted");
        }
        if baseline.snapshot.files.0.get(&span.path) == candidate.snapshot.files.0.get(&span.path)
            && !candidate_selected
                .iter()
                .any(|s| s.path == span.path && s.start <= span.start && s.end >= span.end)
        {
            finding(
                id,
                "unchanged baseline ownership is narrowed by candidate selections",
            );
        }
    }
    for (id, review) in &ownership.review.relationships.0 {
        let external = review
            .external_contract
            .as_ref()
            .is_some_and(|s| !s.trim().is_empty());
        if review.rationale.trim().is_empty() || (review.owned_targets.is_empty() && !external) {
            finding(id, "candidate relationship ownership review is incomplete");
        }
        for target in &review.owned_targets {
            if !ownership
                .replacements
                .0
                .get(target)
                .is_some_and(|s| !s.is_empty())
            {
                finding(
                    id,
                    &format!("candidate relationship owned target is omitted: {target}"),
                );
            }
        }
    }
    // Retain every maintained path, including unclassified and explicitly excluded
    // source. Generated/tool-state trees were already pruned by frozen inventory.
    let paths: std::collections::BTreeSet<_> = [&baseline, &candidate]
        .into_iter()
        .flat_map(|side| {
            side.snapshot
                .files
                .0
                .keys()
                .chain(side.snapshot.unclassified.0.keys())
                .chain(side.snapshot.excluded_files.0.keys())
        })
        .cloned()
        .collect();
    for path in paths {
        let record = |snapshot: &Snapshot| {
            (
                snapshot.files.0.get(&path).cloned(),
                snapshot.unclassified.0.get(&path).cloned(),
                snapshot.excluded_files.0.get(&path).cloned(),
            )
        };
        if record(baseline.snapshot) == record(candidate.snapshot) {
            continue;
        }
        result.changed_paths.push(path.clone());
        if !manifest
            .wider_boundary
            .iter()
            .any(|rule| matches_rule(&path, rule))
        {
            result.outside_boundary.push(path.clone());
        }
        if [&baseline, &candidate]
            .iter()
            .any(|side| side.snapshot.unclassified.0.contains_key(&path))
        {
            finding(
                &path,
                "unclassified changed file requires a newly frozen classification",
            );
        }
        if [&baseline, &candidate]
            .iter()
            .any(|side| side.snapshot.excluded_files.0.contains_key(&path))
        {
            finding(
                &path,
                "changed excluded maintained source requires a newly frozen trial",
            );
        }
        let dependency = [&baseline, &candidate].iter().any(|side| {
            side.snapshot
                .files
                .0
                .get(&path)
                .is_some_and(|f| f.category == crate::model::Category::Dependency)
        });
        if dependency {
            result.dependency_paths.push(path.clone());
            if !evidence
                .dependency_changes
                .0
                .get(&path)
                .is_some_and(|change| {
                    !change.description.trim().is_empty()
                        && change
                            .repository_decision
                            .as_ref()
                            .is_some_and(|s| !s.trim().is_empty())
                })
            {
                finding(
                    &path,
                    "dependency change requires disclosure and repository review decision",
                );
            }
        }
        if [&baseline, &candidate].iter().any(|side| {
            side.snapshot
                .files
                .0
                .get(&path)
                .is_some_and(|f| f.category == crate::model::Category::Production)
        }) && !original
            .spans
            .iter()
            .chain(&candidate_selected)
            .any(|span| span.path == path)
        {
            finding(&path, "unrelated or unreviewed production edit requires a clean candidate or reviewed ownership");
        }
    }
    for (side, selected, accounting) in [
        (&baseline, original.spans, &mut result.baseline),
        (&candidate, candidate_selected, &mut result.candidate),
    ] {
        for span in &selected {
            if !side.snapshot.files.0.contains_key(&span.path) {
                finding(
                    &span.path,
                    "selected implementation has no frozen classification",
                );
            }
        }
        let whole = |paths: Vec<String>| {
            resolve_selections(
                side.root,
                side.snapshot,
                &Records(
                    paths
                        .into_iter()
                        .map(|path| (path.clone(), Selector::WholeFile { path }))
                        .collect(),
                ),
            )
            .map(|resolved| resolved.0.into_values().collect::<Vec<_>>())
        };
        let expanded = whole(
            result
                .changed_paths
                .iter()
                .filter(|path| side.snapshot.files.0.contains_key(*path))
                .cloned()
                .collect(),
        )?;
        accounting.selected = selected.clone();
        accounting.capability = union_spans(selected.into_iter().chain(expanded).collect());
        let wider = whole(
            side.snapshot
                .files
                .0
                .keys()
                .filter(|path| {
                    manifest
                        .wider_boundary
                        .iter()
                        .any(|rule| matches_rule(path, rule))
                })
                .cloned()
                .collect(),
        )?;
        accounting.wider =
            union_spans(accounting.capability.iter().cloned().chain(wider).collect());
        accounting.categories = accounting
            .wider
            .iter()
            .filter_map(|span| {
                side.snapshot
                    .files
                    .0
                    .get(&span.path)
                    .map(|file| (span.path.clone(), file.category))
            })
            .collect();
    }
    Ok(result)
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
    fn symbol(path: &str, name: &str, occurrence: u64, source: &str) -> Selector {
        Selector::Symbol {
            path: path.into(),
            qualified_name: name.into(),
            occurrence: crate::model::Count(occurrence),
            source_digest: digest(source.as_bytes()),
        }
    }
    fn ownership(paths: &[&str]) -> crate::model::EvidenceBundle {
        serde_json::from_value(json!({"schema_version":1,
            "candidate_ownership":{"schema_version":1,"replacements":{"a":paths.iter().map(|path|json!({"kind":"whole_file","path":path})).collect::<Vec<_>>()},
            "review":{"schema_version":1,"rationale":"reviewed complete replacement","relationships":{}}},
            "checks":{},"dependency_changes":{}})).unwrap()
    }
    fn accounted(
        a: &Fixture,
        b: &Fixture,
        m: &Manifest,
        e: &crate::model::EvidenceBundle,
    ) -> ReplacementAccounting {
        account_replacement(
            SourceSide {
                root: &a.0,
                snapshot: &inventory(&a.0, m).unwrap(),
            },
            SourceSide {
                root: &b.0,
                snapshot: &inventory(&b.0, m).unwrap(),
            },
            m,
            e,
        )
        .unwrap()
    }
    #[test]
    fn replacement_accounting_moved_and_fragmented_helpers_expand_symmetrically() {
        let a = Fixture::new();
        let b = Fixture::new();
        let mut m = manifest();
        m.wider_boundary = vec!["src".into()];
        m.categories
            .0
            .insert("outside".into(), Category::Production);
        a.write("src/a.py", "def run():\n    return 1\n");
        for root in [&a, &b] {
            root.write("src/unchanged.py", "unrelated = 100\n");
        }
        b.write(
            "outside/helper.py",
            "def first():\n    return 1\ndef second():\n    return first()\n",
        );
        let result = accounted(
            &a,
            &b,
            &m,
            &ownership(&["outside/helper.py", "outside/helper.py"]),
        );
        assert_eq!(result.changed_paths, vec!["outside/helper.py", "src/a.py"]);
        assert_eq!(result.outside_boundary, vec!["outside/helper.py"]);
        assert_eq!(result.baseline.capability.len(), 1);
        assert_eq!(result.candidate.capability.len(), 1);
        assert_eq!(result.baseline.wider.len(), 2);
        assert_eq!(result.candidate.wider.len(), 2);
        assert!(result.findings.is_empty(), "{:?}", result.findings);
        // Symmetric expansion includes the original full file and the replacement's
        // full support, even when selectors only name one fragment.
        b.write(
            "src/a.py",
            "def run():\n    return 0\ndef hidden():\n    return 2\n",
        );
        let source = fs::read_to_string(b.0.join("src/a.py")).unwrap();
        let mut e = ownership(&["src/a.py"]);
        e.candidate_ownership
            .replacements
            .0
            .insert("a".into(), vec![symbol("src/a.py", "run", 0, &source)]);
        let result = accounted(&a, &b, &m, &e);
        assert!(result
            .candidate
            .capability
            .iter()
            .any(|s| s.path == "src/a.py" && s.end == source.len()));
        assert!(result
            .candidate
            .capability
            .iter()
            .any(|s| s.path == "outside/helper.py"));
        assert!(result
            .findings
            .iter()
            .any(|f| f.explanation.contains("unrelated")));
    }
    #[test]
    fn replacement_accounting_blocks_omissions_narrowing_and_unreviewed_ownership() {
        let a = Fixture::new();
        let b = Fixture::new();
        let m = manifest();
        let source = "def run():\n    return 1\ndef helper():\n    return 2\n";
        for root in [&a, &b] {
            root.write("src/a.py", source);
        }
        let mut e = ownership(&["src/a.py"]);
        e.candidate_ownership
            .replacements
            .0
            .insert("a".into(), vec![symbol("src/a.py", "run", 0, source)]);
        assert!(accounted(&a, &b, &m, &e)
            .findings
            .iter()
            .any(|f| f.explanation.contains("narrow")));
        e.candidate_ownership.replacements.0.clear();
        assert!(accounted(&a, &b, &m, &e)
            .findings
            .iter()
            .any(|f| f.explanation.contains("omitted")));
        e = ownership(&["src/a.py"]);
        e.candidate_ownership.review.rationale.clear();
        assert!(accounted(&a, &b, &m, &e)
            .findings
            .iter()
            .any(|f| f.explanation.contains("review")));
        e = ownership(&["src/a.py"]);
        e.candidate_ownership
            .replacements
            .0
            .insert("unknown".into(), vec![]);
        assert!(accounted(&a, &b, &m, &e)
            .findings
            .iter()
            .any(|f| f.subject == "unknown"));
    }
    #[test]
    fn replacement_accounting_discloses_categories_dependencies_and_blocked_changes() {
        let a = Fixture::new();
        let b = Fixture::new();
        let mut m = manifest();
        m.exclusions.0.insert(
            "private".into(),
            Exclusion {
                schema_version: SchemaVersion::V1,
                rationale: "outside capability".into(),
                kind: ExclusionKind::MaintainedSource,
            },
        );
        for root in [&a, &b] {
            root.write("src/a.py", "pass\n");
            root.write("Cargo.toml", "old");
            root.write("private/hidden.py", "old");
        }
        b.write("Cargo.toml", "new");
        b.write("private/hidden.py", "new");
        b.write("unknown.txt", "new");
        b.write("tests/test_a.py", "assert True\n");
        b.write("settings.json", "{}");
        let result = accounted(&a, &b, &m, &ownership(&["src/a.py"]));
        assert_eq!(result.dependency_paths, vec!["Cargo.toml"]);
        for needle in ["unclassified", "excluded maintained", "dependency"] {
            assert!(
                result
                    .findings
                    .iter()
                    .any(|f| f.explanation.contains(needle)),
                "{needle}: {:?}",
                result.findings
            );
        }
        assert!(result
            .candidate
            .capability
            .iter()
            .any(|s| s.path == "settings.json"));
        assert!(result
            .candidate
            .capability
            .iter()
            .any(|s| s.path == "tests/test_a.py"));
        assert_eq!(
            result.candidate.categories["tests/test_a.py"],
            Category::Tests
        );
        assert_eq!(
            result.candidate.categories["settings.json"],
            Category::Configuration
        );
        let mut disclosed = ownership(&["src/a.py"]);
        disclosed.dependency_changes.0.insert(
            "Cargo.toml".into(),
            crate::model::DependencyChange {
                schema_version: SchemaVersion::V1,
                description: "reviewed installed dependency change".into(),
                repository_decision: Some("repository decision recorded".into()),
            },
        );
        let reviewed = accounted(&a, &b, &m, &disclosed);
        assert_eq!(reviewed.dependency_changes, disclosed.dependency_changes.0);
        assert!(!reviewed
            .findings
            .iter()
            .any(|f| f.explanation.contains("dependency change requires")));
        let snap_a = inventory(&a.0, &m).unwrap();
        let mut snap_b = inventory(&b.0, &m).unwrap();
        snap_b.files.0.get_mut("src/a.py").unwrap().category = Category::Tests;
        let result = account_replacement(
            SourceSide {
                root: &a.0,
                snapshot: &snap_a,
            },
            SourceSide {
                root: &b.0,
                snapshot: &snap_b,
            },
            &m,
            &ownership(&["src/a.py"]),
        )
        .unwrap();
        assert!(result
            .findings
            .iter()
            .any(|f| f.explanation.contains("classification")));
    }
    #[test]
    fn reviewed_selectors_retain_entries_and_union_whole_file_overlap() {
        let root = Fixture::new();
        let source = "@decorator\ndef entry():\n    return 1\n\ndef helper():\n    return 2\n";
        root.write("src/a.py", source);
        let mut m = manifest();
        m.entry_points
            .0
            .insert("entry".into(), symbol("src/a.py", "entry", 0, source));
        m.owned
            .0
            .insert("entry".into(), symbol("src/a.py", "entry", 0, source));
        let resolved = resolve_scope(
            &root.0,
            &inventory(&root.0, &m).unwrap(),
            &m,
            &Records(BTreeMap::new()),
        )
        .unwrap();
        assert_eq!(
            resolved.spans,
            vec![SourceSpan {
                path: "src/a.py".into(),
                start: 0,
                end: source.len()
            }]
        );
        assert_eq!(
            &source[resolved.entry_points.0["entry"].start..resolved.entry_points.0["entry"].end],
            "@decorator\ndef entry():\n    return 1"
        );
        assert_eq!(resolved.review, m.scope_review);
        assert!(resolved.findings.is_empty());
    }
    #[test]
    fn qualified_occurrences_select_complete_nodes_and_reject_stale_or_missing_source() {
        let root = Fixture::new();
        let source = "class A:\n    def run(self):\n        return 'é'\n\ndef run():\n    return 1\ndef run():\n    return 2\n";
        root.write("src/a.py", source);
        let mut m = manifest();
        m.entry_points.0.clear();
        m.owned.0.clear();
        m.owned
            .0
            .insert("method".into(), symbol("src/a.py", "A.run", 0, source));
        m.owned
            .0
            .insert("repeat".into(), symbol("src/a.py", "run", 1, source));
        let snapshot = inventory(&root.0, &m).unwrap();
        let resolved = resolve_scope(&root.0, &snapshot, &m, &Records(BTreeMap::new())).unwrap();
        assert_eq!(
            &source[resolved.owned.0["method"].start..resolved.owned.0["method"].end],
            "def run(self):\n        return 'é'"
        );
        assert_eq!(
            &source[resolved.owned.0["repeat"].start..resolved.owned.0["repeat"].end],
            "def run():\n    return 2"
        );
        for selector in [
            symbol("src/a.py", "run", 2, source),
            symbol("src/a.py", "absent", 0, source),
            symbol("src/a.py", "run", 0, "stale"),
            symbol("src/missing.py", "run", 0, source),
        ] {
            m.owned.0.insert("bad".into(), selector);
            assert!(resolve_scope(&root.0, &snapshot, &m, &Records(BTreeMap::new())).is_err());
        }
        m.owned.0.remove("bad");
        root.write("src/a.py", "changed = True\n");
        assert!(resolve_scope(&root.0, &snapshot, &m, &Records(BTreeMap::new())).is_err());
    }
    #[test]
    fn narrowed_or_unreviewed_scope_is_incomplete_and_locations_survive_review() {
        let root = Fixture::new();
        let source = "def run():\n    unknown()\n\ndef helper():\n    return 1\n";
        root.write("src/a.py", source);
        let mut m = manifest();
        m.owned
            .0
            .insert("a".into(), symbol("src/a.py", "helper", 0, source));
        m.entry_points
            .0
            .insert("entry".into(), symbol("src/a.py", "run", 0, source));
        let unresolved: Records<String, crate::model::Finding> = serde_json::from_value(json!({"src/a.py:2:5":{"schema_version":1,"subject":"src/a.py:2:5","explanation":"dynamic target"}})).unwrap();
        let snap = inventory(&root.0, &m).unwrap();
        let result = resolve_scope(&root.0, &snap, &m, &unresolved).unwrap();
        assert_eq!(result.unresolved, unresolved);
        assert_eq!(result.findings.len(), 2);
        m.owned.0.insert(
            "a".into(),
            Selector::WholeFile {
                path: "src/a.py".into(),
            },
        );
        m.scope_review.relationships = serde_json::from_value(json!({"src/a.py:2:5":{"schema_version":1,"owned_targets":["a"],"external_contract":null,"rationale":"helper owned by selected module"}})).unwrap();
        let result = resolve_scope(&root.0, &snap, &m, &unresolved).unwrap();
        assert_eq!(result.unresolved, unresolved);
        assert!(result.findings.is_empty());
        m.scope_review
            .relationships
            .0
            .get_mut("src/a.py:2:5")
            .unwrap()
            .owned_targets = vec!["omitted".into()];
        assert!(!resolve_scope(&root.0, &snap, &m, &unresolved)
            .unwrap()
            .findings
            .is_empty());
        m.scope_review.rationale.clear();
        assert!(
            resolve_scope(&root.0, &snap, &m, &unresolved)
                .unwrap()
                .findings
                .len()
                >= 2
        );
    }
    #[test]
    fn typed_named_nodes_keep_export_wrappers_and_tsx_bodies() {
        let root = Fixture::new();
        let ts =
            "export class Worker { run(): number { return 1; } }\nexport const invoke = () => 2;\n";
        let tsx = "export const View = () => <div>é</div>;\n";
        root.write("src/a.py", "pass\n");
        root.write("src/a.ts", ts);
        root.write("src/view.tsx", tsx);
        let snap = inventory(&root.0, &manifest()).unwrap();
        let selections = Records(BTreeMap::from([
            ("method".into(), symbol("src/a.ts", "Worker.run", 0, ts)),
            ("arrow".into(), symbol("src/a.ts", "invoke", 0, ts)),
            ("view".into(), symbol("src/view.tsx", "View", 0, tsx)),
        ]));
        let result = resolve_selections(&root.0, &snap, &selections).unwrap();
        assert_eq!(
            &ts[result.0["method"].start..result.0["method"].end],
            "run(): number { return 1; }"
        );
        assert_eq!(
            &ts[result.0["arrow"].start..result.0["arrow"].end],
            "export const invoke = () => 2;"
        );
        assert_eq!(
            &tsx[result.0["view"].start..result.0["view"].end],
            tsx.trim_end()
        );
    }
    #[test]
    fn structural_selector_failures_never_fall_back_to_empty_spans() {
        let root = Fixture::new();
        root.write("src/a.py", "def broken(:\n");
        root.write("src/data.txt", "data");
        fs::write(root.0.join("src/raw.py"), [0xff]).unwrap();
        let snap = inventory(&root.0, &manifest()).unwrap();
        for selector in [
            symbol("src/a.py", "broken", 0, "def broken(:\n"),
            symbol("src/data.txt", "data", 0, "data"),
            Selector::Symbol {
                path: "src/raw.py".into(),
                qualified_name: "raw".into(),
                occurrence: crate::model::Count(0),
                source_digest: digest(&[0xff]),
            },
            Selector::WholeFile {
                path: "src/absent.py".into(),
            },
            Selector::WholeFile {
                path: "../outside.py".into(),
            },
        ] {
            assert!(resolve_selections(
                &root.0,
                &snap,
                &Records(BTreeMap::from([("bad".into(), selector)]))
            )
            .is_err());
        }
        let whole = Records(BTreeMap::from([(
            "raw".into(),
            Selector::WholeFile {
                path: "src/raw.py".into(),
            },
        )]));
        assert_eq!(
            resolve_selections(&root.0, &snap, &whole).unwrap().0["raw"].end,
            1
        );
        let mut excluded = manifest();
        excluded.owned.0.clear();
        excluded.entry_points.0.clear();
        excluded.exclusions.0.insert(
            "src/data.txt".into(),
            Exclusion {
                schema_version: SchemaVersion::V1,
                rationale: "outside ownership".into(),
                kind: ExclusionKind::MaintainedSource,
            },
        );
        let snap = inventory(&root.0, &excluded).unwrap();
        let whole = Records(BTreeMap::from([(
            "data".into(),
            Selector::WholeFile {
                path: "src/data.txt".into(),
            },
        )]));
        assert!(resolve_selections(&root.0, &snap, &whole).is_err());
    }
    #[test]
    fn span_union_counts_nested_partial_and_adjacent_ranges_once_per_file() {
        let span = |path: &str, start, end| SourceSpan {
            path: path.into(),
            start,
            end,
        };
        assert_eq!(
            union_spans(vec![
                span("b", 0, 2),
                span("a", 8, 12),
                span("a", 0, 5),
                span("a", 2, 4),
                span("a", 4, 8),
                span("a", 20, 25)
            ]),
            vec![span("a", 0, 12), span("a", 20, 25), span("b", 0, 2)]
        );
    }
    #[test]
    fn external_contract_review_retains_relationship_without_an_owned_target() {
        let root = Fixture::new();
        root.write("src/a.py", "external()\n");
        let mut m = manifest();
        let unresolved: Records<String, crate::model::Finding> = serde_json::from_value(json!({"src/a.py:1:1":{"schema_version":1,"subject":"src/a.py:1:1","explanation":"external call"}})).unwrap();
        m.scope_review.relationships = serde_json::from_value(json!({"src/a.py:1:1":{"schema_version":1,"owned_targets":[],"external_contract":"installed library API","rationale":"dependency owns implementation"}})).unwrap();
        let snap = inventory(&root.0, &m).unwrap();
        assert!(resolve_scope(&root.0, &snap, &m, &unresolved)
            .unwrap()
            .findings
            .is_empty());
        m.scope_review
            .relationships
            .0
            .get_mut("src/a.py:1:1")
            .unwrap()
            .external_contract = Some(" ".into());
        assert_eq!(
            resolve_scope(&root.0, &snap, &m, &unresolved)
                .unwrap()
                .findings
                .len(),
            1
        );
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
