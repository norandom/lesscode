//! The packs shipped inside the binary. This is the one place in the driver
//! that lists languages, and it does so only by naming the pack files.

use super::pack::{Pack, PackError};
use std::sync::OnceLock;

const SHIPPED: &[&str] = &[
    include_str!("../../../packs/java.json"),
    include_str!("../../../packs/python.json"),
    include_str!("../../../packs/rust.json"),
    include_str!("../../../packs/typescript.json"),
];

/// The text of every shipped pack.
pub fn shipped_texts() -> &'static [&'static str] {
    SHIPPED
}

/// Every shipped pack, parsed once per process.
pub fn shipped() -> Result<&'static [Pack], PackError> {
    static PACKS: OnceLock<Result<Vec<Pack>, PackError>> = OnceLock::new();
    PACKS
        .get_or_init(|| SHIPPED.iter().map(|text| Pack::parse(text)).collect())
        .as_deref()
        .map_err(Clone::clone)
}

/// The shipped pack for `language`.
pub fn embedded(language: &str) -> Result<&'static Pack, PackError> {
    shipped()?
        .iter()
        .find(|pack| pack.language == language)
        .ok_or_else(|| PackError::NotSupported(language.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::driver::Kind;
    use std::collections::BTreeSet;

    #[test]
    fn every_shipped_pack_loads_and_names_a_distinct_language() {
        let packs = shipped().unwrap();
        let names: BTreeSet<_> = packs.iter().map(|p| p.language.as_str()).collect();
        assert_eq!(names.len(), SHIPPED.len());
        for pack in packs {
            assert!(
                pack.rule(Kind::BranchCondition).is_some(),
                "{}",
                pack.language
            );
            assert_eq!(embedded(&pack.language).unwrap().digest, pack.digest);
        }
    }

    #[test]
    fn an_unknown_language_is_not_supported() {
        assert_eq!(
            embedded("klingon").unwrap_err(),
            PackError::NotSupported("klingon".into())
        );
    }

    #[test]
    fn a_directory_pack_must_name_its_language() {
        let dir = std::env::temp_dir().join(format!("lesscode-pack-dir-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("one.json"), SHIPPED[0]).unwrap();
        let first = &shipped().unwrap()[0].language;
        std::fs::write(dir.join(format!("{first}.json")), SHIPPED[0]).unwrap();
        assert!(Pack::from_dir(&dir, first).is_ok());
        assert!(matches!(
            Pack::from_dir(&dir, "one"),
            Err(PackError::Invalid(_))
        ));
        assert!(matches!(
            Pack::from_dir(&dir, "two"),
            Err(PackError::NotSupported(_))
        ));
        assert!(matches!(
            Pack::from_dir(&dir, "../one"),
            Err(PackError::NotSupported(_))
        ));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
