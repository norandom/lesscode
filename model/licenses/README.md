# License texts kept for packages that publish none

Five packages of `model/Cargo.lock` hold no license file in their published
archive. Their texts are kept here, fetched on 2026-11-16 from the source
commit recorded in each crate's `.cargo_vcs_info.json`.
`scripts/license-inventory.ps1` puts them into `THIRD_PARTY_NOTICES.txt`.

| Package | Files here | Source commit |
| --- | --- | --- |
| `jni` 0.22.4, `jni-macros` 0.22.4 | `jni-MIT.txt`, `jni-APACHE.txt` | [jni-rs/jni-rs `5ae9458`](https://github.com/jni-rs/jni-rs/tree/5ae9458a4ec44c5318f37ddc7569c1d4ae8a69e7) (`jni`); the two files are identical at `33045a1` (`jni-macros`) |
| `jni-sys-macros` 0.4.1 | `jni-sys-macros-MIT.txt`, `jni-sys-macros-APACHE.txt` | [jni-rs/jni-sys `64d77b7`](https://github.com/jni-rs/jni-sys/tree/64d77b7a5f119d7b55b4e2c169a4668067ff59e6) |
| `rustls-platform-verifier-android` 0.2.0 | `rustls-platform-verifier-android-MIT.txt`, `rustls-platform-verifier-android-APACHE.txt` | [rustls/rustls-platform-verifier `252e251`](https://github.com/rustls/rustls-platform-verifier/tree/252e25161a91af476cbea620e29d277592d27ec7) |
| `r-efi` 6.0.0 | `r-efi-README.txt`, `r-efi-AUTHORS.txt` | [r-efi/r-efi `7e1b032`](https://github.com/r-efi/r-efi/tree/7e1b0322d31d625f81a5656096330934f9cd835d) |

`r-efi` has no license file in its repository either. Its README states the
license (MIT OR Apache-2.0 OR LGPL-2.1-or-later) and points to AUTHORS, so
those two files stand in.

The texts for `ast-grep-core`, `ast-grep-language`, `tree-sitter-java` and
`tree-sitter-typescript`, which reach this crate through `lesscode-core`, are
kept once, under `native/licenses/`.

These five packages are locked for Android and UEFI targets. None of them is
compiled into a released binary; the inventory lists every locked record.
