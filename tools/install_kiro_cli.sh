#!/usr/bin/env bash
# Install the pinned kiro-cli release binary (openidea-kiro) into tools/bin, which is not committed.
# kiro-cli audits the specs under .kiro/specs and the code; it is part of the guard for refactoring steps
# (.kiro/specs/gradual-refactoring-steps). The digests are those of SHA256SUMS.txt of the release; a version
# change needs new digests:
#   gh release download vVERSION --repo norandom/openidea-kiro --pattern SHA256SUMS.txt
set -euo pipefail

version="1.1.0"
dir="${1:-$(cd "$(dirname "$0")" && pwd)/bin}"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64 | Linux-amd64)
    asset="kiro-cli-v$version-x86_64-unknown-linux-gnu.tar.gz"
    digest="1391e3658823fb18a1269f62c85e905e3033dbaf92d59501110124731776669d"
    ;;
  Darwin-x86_64)
    asset="kiro-cli-v$version-x86_64-apple-darwin.tar.gz"
    digest="c98d06c307900fa0e1f738160fe359128451d90e57bf3a315413873042708134"
    ;;
  Darwin-arm64 | Darwin-aarch64)
    asset="kiro-cli-v$version-aarch64-apple-darwin.tar.gz"
    digest="62eebacb6ae3252e6260a17e86451261e467c09d31a2d757af3f700c4f35081a"
    ;;
  MINGW*-x86_64 | MSYS*-x86_64)
    asset="kiro-cli-v$version-x86_64-pc-windows-msvc.zip"
    digest="7edf626dae1990a00db014279922b6f1d514b94b6c56f7dda7470b713771d2bc"
    ;;
  *)
    # The release has no build for this platform (for example Linux on aarch64).
    echo "install_kiro_cli: no kiro-cli $version release for $(uname -s) $(uname -m)" >&2
    exit 2
    ;;
esac

if [ -x "$dir/kiro-cli" ] && [ "$("$dir/kiro-cli" --version)" = "kiro-cli $version" ]; then
  echo "kiro-cli $version is already in $dir"
  exit 0
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
curl -fsSL -o "$work/$asset" "https://github.com/norandom/openidea-kiro/releases/download/v$version/$asset"
if command -v sha256sum >/dev/null 2>&1; then
  found="$(sha256sum "$work/$asset" | cut -d' ' -f1)"
else
  found="$(shasum -a 256 "$work/$asset" | cut -d' ' -f1)"
fi
if [ "$found" != "$digest" ]; then
  echo "install_kiro_cli: digest of $asset is $found, expected $digest" >&2
  exit 1
fi
mkdir -p "$dir"
case "$asset" in
  *.zip) unzip -q -o "$work/$asset" -d "$work/x" ;;
  *) mkdir -p "$work/x" && tar -xzf "$work/$asset" -C "$work/x" ;;
esac
bin="$(find "$work/x" -type f \( -name kiro-cli -o -name kiro-cli.exe \) | head -n 1)"
cp "$bin" "$dir/"
chmod +x "$dir/$(basename "$bin")"
echo "Installed $("$dir/$(basename "$bin")" --version) to $dir"
