//! What a step does to the functions of a file: which changed, which call
//! the changed ones, and which function a planted fault belongs to.
//!
//! Everything is matched by name. A caller is a function whose tokens hold
//! the name of a changed function directly before an opening parenthesis;
//! that can name a function too many and cannot see calls through values.

use crate::adapters::driver::functions::{functions_of, Function};
use crate::adapters::driver::measure::{tokens, Token};
use crate::adapters::driver::pack::Pack;
use std::collections::{BTreeMap, BTreeSet};

/// Where faults outside every function are counted.
pub const OUTSIDE: &str = "(outside any function)";

type At = (usize, usize);

/// The functions of one file at one revision, with the file's tokens.
pub struct FileFunctions {
    pub functions: Vec<Function>,
    tokens: Vec<Token>,
}

/// Faults of one function by what the tests made of them.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Tally {
    pub noticed: usize,
    pub unnoticed: usize,
    /// Invalid, timed out or not classed.
    pub other: usize,
}

fn span(f: &Function) -> (At, At) {
    ((f.start_line, f.start_column), (f.end_line, f.end_column))
}

impl FileFunctions {
    pub fn read(pack: &Pack, file: &str, text: &str) -> Result<FileFunctions, String> {
        Ok(FileFunctions {
            functions: functions_of(pack, file, text)?,
            tokens: tokens(pack, file, text)?,
        })
    }

    fn tokens_of(&self, f: &Function) -> Vec<&str> {
        let (from, to) = span(f);
        let inside = |(at, _): &&Token| *at >= from && *at < to;
        let texts = self.tokens.iter().filter(inside);
        texts.map(|(_, text)| text.as_str()).collect()
    }

    /// Per name, the token sequences of the functions that carry it, in
    /// source order. Functions without a name are part of the function
    /// around them and are not listed.
    pub fn bodies(&self) -> BTreeMap<&str, Vec<Vec<&str>>> {
        let mut bodies: BTreeMap<&str, Vec<Vec<&str>>> = BTreeMap::new();
        for f in self.functions.iter().filter(|f| !f.anonymous) {
            bodies.entry(&f.name).or_default().push(self.tokens_of(f));
        }
        bodies
    }

    /// Tokens of each named function, summed over functions of one name.
    pub fn sizes(&self) -> BTreeMap<&str, usize> {
        let mut sizes = BTreeMap::new();
        for (name, bodies) in self.bodies() {
            sizes.insert(name, bodies.iter().map(Vec::len).sum());
        }
        sizes
    }

    /// The innermost named function around a position, else the innermost
    /// function, else `OUTSIDE`.
    pub fn owner(&self, at: At) -> &str {
        let around = |f: &&Function| span(f).0 <= at && at < span(f).1;
        let all = || self.functions.iter().filter(around);
        let innermost_named = all().filter(|f| !f.anonymous).max_by_key(|f| span(f).0);
        let innermost = innermost_named.or_else(|| all().max_by_key(|f| span(f).0));
        innermost.map_or(OUTSIDE, |f| f.name.as_str())
    }

    /// The named functions that call one of `names`, themselves excluded.
    pub fn callers_of(&self, names: &BTreeSet<String>) -> BTreeSet<String> {
        let mut callers = BTreeSet::new();
        for pair in self.tokens.windows(2) {
            let ((at, callee), (_, next)) = (&pair[0], &pair[1]);
            let caller = self.owner(*at);
            // The name in the function's own header is not a call.
            if next == "(" && names.contains(callee) && caller != callee && caller != OUTSIDE {
                callers.insert(caller.to_owned());
            }
        }
        callers.retain(|c| !names.contains(c));
        callers
    }

    /// Count faults, given as position and class, per owning function.
    pub fn tally<'a>(
        &self,
        faults: impl IntoIterator<Item = (At, &'a str)>,
    ) -> BTreeMap<String, Tally> {
        let mut table: BTreeMap<String, Tally> = BTreeMap::new();
        for (at, class) in faults {
            let row = table.entry(self.owner(at).to_owned()).or_default();
            match class {
                "killed" => row.noticed += 1,
                "survived" => row.unnoticed += 1,
                _ => row.other += 1,
            }
        }
        table
    }
}

/// The names of the functions whose tokens differ between the two
/// revisions, or that exist in one of them only.
pub fn changed(base: &FileFunctions, step: &FileFunctions) -> BTreeSet<String> {
    let (before, after) = (base.bodies(), step.bodies());
    let names: BTreeSet<&str> = before.keys().chain(after.keys()).copied().collect();
    let differs = |name: &&str| before.get(*name) != after.get(*name);
    names
        .into_iter()
        .filter(differs)
        .map(str::to_owned)
        .collect()
}
