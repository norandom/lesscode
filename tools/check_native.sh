#!/usr/bin/env bash
# Shared by native Actions runners and the Dagger Linux container.
set -euo pipefail
evidence="${1:?Pass an external evidence directory}"
mkdir -p "$evidence"
failed=0
run_check() {
  local label="$1"
  shift
  set +e
  "$@" 2>&1 | tee "$evidence/$label.log"
  local statuses=("${PIPESTATUS[@]}")
  set -e
  local status=${statuses[0]}
  if [ "${statuses[1]}" -ne 0 ]; then status=${statuses[1]}; fi
  printf '%s\n' "$status" > "$evidence/$label.exit-code"
  if [ "$status" -ne 0 ]; then failed=1; fi
}
printf '%s\n' "${GITHUB_SHA:-local-working-tree}" > "$evidence/source-revision.txt"
run_check platform uname -a
run_check rustc rustc +1.98.1 --version --verbose
run_check cargo cargo +1.98.1 --version --verbose
run_check nextest cargo +1.98.1 nextest --version
run_check fmt cargo +1.98.1 fmt --manifest-path native/Cargo.toml --all -- --check
run_check clippy cargo +1.98.1 clippy --locked --manifest-path native/Cargo.toml --all-targets -- -D warnings
run_check test cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml --test-threads 4 --no-fail-fast
run_check doctest cargo +1.98.1 test --locked --manifest-path native/Cargo.toml --doc
run_check build cargo +1.98.1 build --locked --manifest-path native/Cargo.toml
run_check help cargo +1.98.1 run --locked --manifest-path native/Cargo.toml -- --help
printf '%s\n' "$failed" > "$evidence/overall.exit-code"
exit "$failed"
