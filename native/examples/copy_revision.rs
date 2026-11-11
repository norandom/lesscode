//! Copy a directory of a repository as it is at a revision, by reading its
//! blobs: `copy_revision REPO REVISION ROOT DEST`. Used for the evidence of
//! gradual-refactoring-steps task 1.2 until the step commands exist.

use lesscode_core::step::revisions::{copy_tree, resolve};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [repo, revision, root, dest] = args.as_slice() else {
        eprintln!("usage: copy_revision REPO REVISION ROOT DEST");
        std::process::exit(3);
    };
    let done = resolve(Path::new(repo), revision).and_then(|hash| {
        copy_tree(Path::new(repo), &hash, root, Path::new(dest)).map(|c| (hash, c))
    });
    match done {
        Ok((hash, copied)) => {
            println!("{hash}: {} files", copied.files);
            for (path, why) in copied.skipped {
                println!("skipped {path}: {why}");
            }
        }
        Err(why) => {
            eprintln!("{why}");
            std::process::exit(3);
        }
    }
}
