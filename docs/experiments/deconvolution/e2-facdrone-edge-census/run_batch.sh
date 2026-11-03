#!/usr/bin/env bash
# E2 batched full run (plan amendment 6). Usage: run_batch.sh RUN BATCH
#   RUN   = run1 | run2 | base   (base = unrecorded baseline: no recorder plugin, no E2 env, same shim and markers)
#   BATCH = root-unit | root-contracts | root-property | root-rest | research | reporting | dash | kernel | factor-kit | factor-comm-term-structure | factor-short-reversal
# One batch per invocation; never run two concurrently. N=2 workers.
set -euo pipefail
RUN=$1; B=$2; N=${N:-2}
ED=D:/Source/lesscode/docs/experiments/deconvolution/e2-facdrone-edge-census
S=D:/Source/lesscode/.runs/e2-facdrone/source
V=D:/Source/lesscode/.runs/e2-facdrone/venv/Scripts/python.exe
MAN=D:/Source/lesscode/docs/experiments/constraint-modeler/facdrone-discovery/run-02/source-manifest.json
MK="not port_baseline and not live_broker"
ENVS="--env PYTHONPATH=$ED --env OPENBLAS_NUM_THREADS=1 --env OMP_NUM_THREADS=1 --env MKL_NUM_THREADS=1"
PLUGINS="-p e2_winshim -p e2_recorder"
if [ "$RUN" = base ]; then
  PLUGINS="-p e2_winshim"
else
  ENVS="$ENVS --env E2_SOURCE_ROOT=$S --env E2_MANIFEST=$MAN --env COVERAGE_CORE=sysmon --env E2_OUT=$ED/results/$RUN/$B"
fi
PYT="uv.exe run --no-project --python $V python -m pytest -q -p no:cacheprovider $PLUGINS -n $N --dist loadgroup --timeout=600"
cd "$ED"
case "$B" in
  root-unit)      CWD=$S; T="tests/unit" ;;
  root-contracts) CWD=$S; T="tests/contracts" ;;
  root-property)  CWD=$S; T="tests/property" ;;
  root-rest)      CWD=$S; T="tests --ignore=tests/unit --ignore=tests/contracts --ignore=tests/property" ;;
  research)       CWD=$S/packages/facdrone-research; T="--ignore=tests/test_lifecycle_full_loop_db.py" ;;
  *)              CWD=$S/packages/facdrone-$B; T="" ;;
esac
mkdir -p "results/$RUN"
uv.exe run --no-project --python 3.13 python receipt.py run "$RUN-$B" --cwd "$CWD" $ENVS -- \
  $PYT $T -m "$MK" --junitxml="$ED/results/$RUN/$B.xml"
