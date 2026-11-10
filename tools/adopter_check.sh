#!/usr/bin/env bash
# Adopter check: install lesscode-core from a GitHub release the way the README
# says, then run the documented commands against this repository's sources.
# Runs inside the containers defined in tools/adopter-check.compose.yaml; the
# repository is mounted read-only at /repo and receipts stay in the container.
#
# Environment:
#   LESSCODE_VERSION   release to install (default: latest)
#   INSTALLER          "published" (default) pipes the release's install.sh;
#                      "local" runs /repo/scripts/install.sh instead
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive
apt-get update -qq > /dev/null
apt-get install -y -qq --no-install-recommends ca-certificates curl > /dev/null

echo "== system: $(uname -sm), glibc $(ldd --version | head -n 1 | awk '{print $NF}')"

if [ "${INSTALLER:-published}" = local ]; then
  sh /repo/scripts/install.sh
else
  curl -fsSL https://github.com/norandom/lesscode/releases/latest/download/install.sh | sh
fi
export PATH="$HOME/.local/bin:$PATH"

out="$(mktemp -d)"
failed=0
# expect LABEL WANTED_EXIT COMMAND...
expect() {
  local label="$1" want="$2" got=0
  shift 2
  "$@" > "$out/$label.log" 2>&1 || got=$?
  if [ "$got" -eq "$want" ]; then
    echo "ok    $label (exit $got)"
  else
    echo "FAIL  $label: exit $got, expected $want"
    sed 's/^/      /' "$out/$label.log" | head -n 12
    failed=1
  fi
}
field() { sed -n "s/^ *\"$2\": *\(.*[^,]\),\{0,1\}$/\1/p" "$1" | head -n 1; }

cd /repo
echo "== $(lesscode-core --version)"

expect density-lifecycle 0 lesscode-core deconvolute density --lang rust \
  --baseline native/adapters/rust/baseline_lifecycle.rs \
  --candidate native/adapters/rust/candidate_lifecycle.rs --out "$out/density.json"
expect density-identical 1 lesscode-core deconvolute density --lang rust \
  --baseline native/src/density.rs --candidate native/src/density.rs
expect analyze-rust 0 lesscode-core deconvolute analyze --lang rust \
  --source native/src/budgets.rs --out "$out/analyze-rust.json"
expect analyze-typescript 0 lesscode-core deconvolute analyze --lang typescript \
  --source native/adapters/typescript/candidate_degradation.ts --out "$out/analyze-ts.json"
expect analyze-java 0 lesscode-core deconvolute analyze --lang java \
  --source native/adapters/java/ProbeMain.java --out "$out/analyze-java.json"
expect budgets 0 lesscode-core budgets --config native/complexity-budgets.json \
  --root native/src --out "$out/budgets.json"
expect budgets-wrong-root 1 lesscode-core budgets --config native/complexity-budgets.json \
  --root native --out "$out/budgets-wrong.json"

echo "== density: $(field "$out/density.json" baseline_lines) -> $(field "$out/density.json" candidate_lines) lines," \
  "$(field "$out/density.json" baseline_stmts) -> $(field "$out/density.json" candidate_stmts) statements," \
  "verdict $(field "$out/density.json" verdict)"

# Known gaps from R55. Reported, not asserted, because the answer depends on the
# installed release: 1.0.1 exits 0 in both cases, later releases fail loudly.
py_exit=0
lesscode-core deconvolute analyze --lang python --source native/shim/recording_shim.py \
  --out "$out/analyze-python.json" > /dev/null 2>&1 || py_exit=$?
echo "== python analyze: exit $py_exit (2 = not implemented and nothing written; 0 = the 1.0.1 stub)"
mut_exit=0
lesscode-core deconvolute mutate --lang rust --source native/src/density.rs \
  --out-dir "$out/mutants" --out "$out/mutate.json" > /dev/null 2>&1 || mut_exit=$?
echo "== rust mutate on density.rs: exit $mut_exit, mutants_generated=$(field "$out/mutate.json" mutants_generated) (exit 1 = no operator matched; 1.0.1 exits 0)"

if [ "$failed" -ne 0 ]; then
  echo "ADOPTER CHECK FAILED"
  exit 1
fi
echo "ADOPTER CHECK PASSED"
