#!/usr/bin/env bash
# Install the pinned cargo-nextest release binary instead of compiling it.
# Shared by native Actions runners and the Dagger Linux container.
#
# The archive digests below are the SHA-256 values published for the
# cargo-nextest 0.9.143 release assets. A version change needs new digests:
#   gh api repos/nextest-rs/nextest/releases/tags/cargo-nextest-VERSION \
#     --jq '.assets[] | select(.name | endswith(".tar.gz")) | "\(.name) \(.digest)"'
set -euo pipefail

version="0.9.143"

case "$(uname -s)" in
  Linux)
    case "$(uname -m)" in
      x86_64 | amd64)
        target="x86_64-unknown-linux-gnu"
        digest="66786b9abe23920d022a182d1416b1bbc8130dd4872a9553d76985a1708dcd1e"
        ;;
      aarch64 | arm64)
        target="aarch64-unknown-linux-gnu"
        digest="2a64b3566a92508550a7ab29c3e8db25472ca37730ecb4d22100b6aa440c2a68"
        ;;
      *) echo "install_nextest: unsupported Linux CPU $(uname -m)" >&2; exit 1 ;;
    esac
    ;;
  Darwin)
    target="universal-apple-darwin"
    digest="4830d430411148d17602a75cc880bfb4dc8dac153dea59a48a2ef4cc93577f07"
    ;;
  MINGW* | MSYS* | CYGWIN*)
    target="x86_64-pc-windows-msvc"
    digest="c42a1dbde532da06dc9b4a43d44fd0ce668b836c2ab7388410f10ff9834476a2"
    ;;
  *) echo "install_nextest: unsupported system $(uname -s)" >&2; exit 1 ;;
esac

archive="cargo-nextest-$version-$target.tar.gz"
url="https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-$version/$archive"
bin_dir="${CARGO_HOME:-$HOME/.cargo}/bin"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

curl -fsSL --retry 3 -o "$work/$archive" "$url"
if command -v sha256sum > /dev/null 2>&1; then
  actual="$(sha256sum "$work/$archive" | awk '{print $1}')"
else
  actual="$(shasum -a 256 "$work/$archive" | awk '{print $1}')"
fi
if [ "$actual" != "$digest" ]; then
  echo "install_nextest: digest mismatch for $archive: expected $digest, got $actual" >&2
  exit 1
fi

mkdir -p "$bin_dir"
tar -xzf "$work/$archive" -C "$bin_dir"
echo "Installed cargo-nextest $version ($target) to $bin_dir"
