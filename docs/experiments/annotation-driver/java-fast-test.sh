#!/bin/sh
# Fast test run for a Java scope inside the long-lived worker container:
# compile only the given source files, then run one test class with surefire.
# The full Maven "test" phase recompiles the module and costs about 20 s per
# run; this takes a few seconds. Usage: java-fast-test.sh TEST_CLASS FILE...
set -eu
test_class="$1"
shift
skips="-Drat.skip=true -Dcheckstyle.skip=true -Dspotbugs.skip=true -Dpmd.skip=true -Djapicmp.skip=true -Djacoco.skip=true -Danimal.sniffer.skip=true"

# Once per working copy: compile everything, tests included.
if [ ! -d target/test-classes ] || [ ! -d target/classes ]; then
    # shellcheck disable=SC2086
    mvn -o -q -B $skips -DskipTests test-compile
fi

# Every jar Maven has already resolved; javac only opens the ones it needs.
jars="$(find /root/.m2/repository -name '*.jar' | tr '\n' ':')"
javac -nowarn -proc:none -encoding UTF-8 -d target/classes -cp "target/classes:$jars" "$@"

# shellcheck disable=SC2086
exec mvn -o -q -B $skips -Dsurefire.failIfNoSpecifiedTests=false -Dtest="$test_class" surefire:test
