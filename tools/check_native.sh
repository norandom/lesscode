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
# The core stays free of models (spec model-seam, task 7.4). The suites below, the step flow's and
# the driver's among them, run only where no lesscode-model can be found: LESSCODE_MODEL_BIN unset,
# none beside the core's or a test's executable, none on the search path (requirements 7.1, 7.2).
no_model_here() {
  local debug="${CARGO_TARGET_DIR:-native/target}/debug" found
  found="${LESSCODE_MODEL_BIN+LESSCODE_MODEL_BIN=$LESSCODE_MODEL_BIN }$(command -v lesscode-model
    ls "$debug"/lesscode-model{,.exe} "$debug"/deps/lesscode-model{,.exe} 2>/dev/null)"
  if [ -n "$found" ]; then
    printf 'lesscode-model is present where the core is tested: %s\n' "$found" >&2
    return 1
  fi
  echo "No lesscode-model: LESSCODE_MODEL_BIN unset, none in $debug or its deps, none on the search path"
}
run_check separation-suites no_model_here
run_check test cargo +1.98.1 nextest run --locked --manifest-path native/Cargo.toml --test-threads 4 --no-fail-fast
run_check doctest cargo +1.98.1 test --locked --manifest-path native/Cargo.toml --doc
run_check build cargo +1.98.1 build --locked --manifest-path native/Cargo.toml
run_check help cargo +1.98.1 run --locked --manifest-path native/Cargo.toml -- --help
# The core links no client library: genai is the seam's, reqwest and hyper are the HTTP client and
# implementation it brings, ureq was the core's own until the seam, and lesscode-model is the seam.
no_client_in_lock() {
  if grep -nE '^name = "(genai|reqwest|hyper|ureq|lesscode-model)"' native/Cargo.lock; then
    echo "native/Cargo.lock holds a client library; the core reaches a model only by starting lesscode-model" >&2
    return 1
  fi
  echo "native/Cargo.lock holds none of genai, reqwest, hyper, ureq, lesscode-model"
}
run_check separation-lock no_client_in_lock
# Everything that needs the client library runs in the one job that sets LESSCODE_CHECK_SEAM (Linux,
# through Dagger), in one target directory, so that the library is compiled in one job only.
if [ "${LESSCODE_CHECK_SEAM:-}" = 1 ]; then
  seam_target="${LESSCODE_SEAM_TARGET_DIR:-model/target}"
  # The seam itself (tasks 7.4 and 8.1): format, lints, its suite against a stub endpoint (no model
  # is called), the scan that its source names no model and holds no address (requirements 1.8, 2.8)
  # under its own name, and its complexity budgets, measured by the core built above.
  run_check model-fmt cargo +1.98.1 fmt --manifest-path model/Cargo.toml --all -- --check
  run_check model-clippy cargo +1.98.1 clippy --locked --manifest-path model/Cargo.toml --target-dir "$seam_target" \
    --all-targets -- -D warnings
  run_check model-test cargo +1.98.1 nextest run --locked --manifest-path model/Cargo.toml --target-dir "$seam_target" \
    --test-threads 4 --no-fail-fast
  run_check separation-model-source cargo +1.98.1 nextest run --locked --manifest-path model/Cargo.toml \
    --target-dir "$seam_target" --test call --no-fail-fast -E 'test(=the_source_names_no_model_and_holds_no_address)'
  run_check model-doctest cargo +1.98.1 test --locked --manifest-path model/Cargo.toml --target-dir "$seam_target" --doc
  run_check model-budgets cargo +1.98.1 run --locked --manifest-path native/Cargo.toml -- budgets \
    --config model/complexity-budgets.json --root model/src --out "$evidence/model-budgets.json"
  # Research tooling beside the product: the guide optimiser, which depends on the seam. Its dry run
  # (a scripted stand-in, no model call) reads the guide and the core's packs and fixtures.
  optimizer=tools/guide-optimizer/Cargo.toml
  run_check optimizer-fmt cargo +1.98.1 fmt --manifest-path "$optimizer" -- --check
  run_check optimizer-clippy cargo +1.98.1 clippy --locked --manifest-path "$optimizer" --target-dir "$seam_target" \
    --all-targets -- -D warnings
  for engine in gepa mipro; do
    run_check "optimizer-dry-run-$engine" cargo +1.98.1 run --locked --manifest-path "$optimizer" \
      --target-dir "$seam_target" -- \
      --dry-run --engine "$engine" --max-metric-calls 24 --repo . --out-dir "$evidence/optimizer-dry-run-$engine"
  done
fi
printf '%s\n' "$failed" > "$evidence/overall.exit-code"
exit "$failed"
