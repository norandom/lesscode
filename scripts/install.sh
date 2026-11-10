#!/bin/sh
# Install the lesscode-core binary from a GitHub release (Linux and macOS).
#
#   curl -fsSL https://github.com/norandom/lesscode/releases/latest/download/install.sh | sh
#
# Environment:
#   LESSCODE_VERSION      release to install, for example 1.0.1 (default: latest)
#   LESSCODE_INSTALL_DIR  directory for the binary (default: $HOME/.local/bin)
#
# The script downloads one archive and its SHA-256 file, verifies the digest,
# and copies the binary. It does not use sudo and does not edit shell profiles.
set -eu

repo="norandom/lesscode"
version="${LESSCODE_VERSION:-latest}"
install_dir="${LESSCODE_INSTALL_DIR:-$HOME/.local/bin}"
notice_dir="${XDG_DATA_HOME:-$HOME/.local/share}/lesscode-core"

fail() {
    printf 'install.sh: %s\n' "$1" >&2
    exit 1
}

case "$(uname -s)" in
    Linux) os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *) fail "unsupported system $(uname -s); on Windows use install.ps1" ;;
esac
case "$(uname -m)" in
    x86_64 | amd64) arch="x86_64" ;;
    aarch64 | arm64) arch="aarch64" ;;
    *) fail "unsupported CPU $(uname -m); build from source instead" ;;
esac

name="lesscode-core-$arch-$os"
archive="$name.tar.gz"
if [ "$version" = "latest" ]; then
    base="https://github.com/$repo/releases/latest/download"
else
    base="https://github.com/$repo/releases/download/v${version#v}"
fi

fetch() {
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL --retry 3 -o "$2" "$1"
    elif command -v wget > /dev/null 2>&1; then
        wget -q -O "$2" "$1"
    else
        fail "curl or wget is required"
    fi
}

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT INT TERM

printf 'Downloading %s (%s)\n' "$archive" "$version"
fetch "$base/$archive" "$work/$archive" || fail "download failed: $base/$archive"
fetch "$base/$archive.sha256" "$work/$archive.sha256" || fail "download failed: $base/$archive.sha256"

expected="$(awk '{print $1; exit}' "$work/$archive.sha256")"
if command -v sha256sum > /dev/null 2>&1; then
    actual="$(sha256sum "$work/$archive" | awk '{print $1}')"
elif command -v shasum > /dev/null 2>&1; then
    actual="$(shasum -a 256 "$work/$archive" | awk '{print $1}')"
else
    fail "sha256sum or shasum is required to verify the download"
fi
[ -n "$expected" ] && [ "$expected" = "$actual" ] ||
    fail "checksum mismatch for $archive: expected $expected, got $actual"

tar -xzf "$work/$archive" -C "$work"
mkdir -p "$install_dir" "$notice_dir"
cp "$work/$name/lesscode-core" "$install_dir/lesscode-core"
chmod 755 "$install_dir/lesscode-core"
cp "$work/$name/LICENSE" "$work/$name/LICENSES.md" "$work/$name/THIRD_PARTY_NOTICES.txt" "$notice_dir/"

installed="$("$install_dir/lesscode-core" --version)" ||
    fail "the installed binary does not run on this system; build from source instead"
printf 'Installed %s to %s\n' "$installed" "$install_dir/lesscode-core"
printf 'License notices: %s\n' "$notice_dir"

case ":$PATH:" in
    *":$install_dir:"*) ;;
    *) printf 'Add %s to PATH to run lesscode-core by name; the Git hook needs that.\n' "$install_dir" ;;
esac
