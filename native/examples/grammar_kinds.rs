//! Research tooling, not part of the product: print the named node kinds of
//! a compiled-in grammar, one per line, sorted.
//!
//!     cargo run --example grammar_kinds -- rust
//!
//! Used by the Slice B experiment that gives a pack author the grammar's
//! node kinds as input (R76).

use ast_grep_core::tree_sitter::LanguageExt;
use lesscode_core::adapters::driver::tree::grammar;
use std::collections::BTreeSet;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(name) = std::env::args().nth(1) else {
        eprintln!("usage: grammar_kinds LANGUAGE");
        return ExitCode::from(3);
    };
    let language = match grammar(&name) {
        Ok(language) => language.get_ts_language(),
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(3);
        }
    };
    let mut kinds = BTreeSet::new();
    for id in 0..language.node_kind_count() as u16 {
        if language.node_kind_is_named(id) && language.node_kind_is_visible(id) {
            if let Some(kind) = language.node_kind_for_id(id) {
                kinds.insert(kind.to_string());
            }
        }
    }
    for kind in kinds {
        println!("{kind}");
    }
    ExitCode::SUCCESS
}
