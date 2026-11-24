#!/bin/sh
# run-dashboard task 8.3: the listener and the opener of `step plan --open` on Linux.
# Builds lesscode-core and the toy test command, runs a plan of a toy project with a stand-in
# `xdg-open` that reads what a browser would read, and says what it found in /out/result.txt.
set -eu

cargo build --locked --bin lesscode-core --example toy_check > /out/build.log 2>&1
core=/target/debug/lesscode-core
toy=/target/debug/examples/toy_check

mkdir -p /tmp/t/project/src /tmp/bin
printf 'def step(state, a):\n    if a > 3:\n        raise ValueError("big")\n    state.total = a\n    return a\n\n\ndef helper(state, b):\n    state.count = b\n    return b\n' > /tmp/t/project/src/app.py
cat > /tmp/t/run.json <<EOF
{"language": "python", "source_root": "project", "files": ["src/app.py"],
 "test": ["$toy", "--file", "src/app.py", "--expect", "if a > 3:"],
 "timeout_floor_s": 10, "timeout_ceiling_s": 60, "workers": 2}
EOF

# The stand-in: what it was given, the page, the first of the stream, and the listener's answer
# to someone who does not know the secret.
cat > /tmp/bin/xdg-open <<'EOF'
#!/bin/sh
echo "$1" >> /out/opened.txt
curl -s -m 5 -D /out/page.headers -o /out/page.html "$1"
curl -s -m 2 -N "${1}events" | head -c 2000 > /out/events.txt
with="${1%/}"
curl -s -m 2 -o /dev/null -w '%{http_code}\n' "${with%/*}/" > /out/without-secret.code
EOF
chmod +x /tmp/bin/xdg-open

# 1. With the stand-in on the path.
set +e
PATH="/tmp/bin:$PATH" "$core" step plan --config /tmp/t/run.json --work /tmp/t/work --out /tmp/t/plan.json --open > /out/plan.log 2>&1
echo $? > /out/exit-code
# The stand-in is not waited for by the tool; give it the time its own limits allow.
sleep 6
cp /tmp/t/plan.html /out/plan.html 2> /dev/null

# 2. Without any opener: this image has no xdg-open. The run must be the same.
"$core" step plan --config /tmp/t/run.json --work /tmp/t/work2 --out /tmp/t/plan2.json --open > /out/no-opener.log 2>&1
echo $? > /out/no-opener.exit-code
set -e

fails=0
say() { if [ "$1" = ok ]; then echo "PASS  $2"; else echo "FAIL  $2"; fails=$((fails + 1)); fi >> /out/result.txt; }
check() { if eval "$2" > /dev/null 2>&1; then say ok "$1"; else say no "$1"; fi; }
: > /out/result.txt
check "the run with the stand-in ends with 0" '[ "$(cat /out/exit-code)" = 0 ]'
check "the opener was started once" '[ "$(wc -l < /out/opened.txt)" = 1 ]'
check "on an address of this machine with a secret" 'grep -Eq "^http://127\.0\.0\.1:[0-9]+/[0-9a-f]{32}/$" /out/opened.txt'
check "the page is served" 'head -1 /out/page.headers | grep -q " 200"'
check "the page is the run page" 'grep -q "lesscode" /out/page.html'
check "the stream carries a snapshot" 'grep -q "\"schema\"" /out/events.txt'
check "without the secret nothing is served" '[ "$(cat /out/without-secret.code)" != 200 ]'
check "the report is written at the end" '[ -s /out/plan.html ]'
check "the tool printed where the run is shown" 'grep -q "The run is shown at http://127.0.0.1:" /out/plan.log'
check "without an opener the run ends with 0" '[ "$(cat /out/no-opener.exit-code)" = 0 ]'
check "and says that the browser could not be started" 'grep -qi "xdg-open" /out/no-opener.log'
uname -srm >> /out/result.txt
"$core" --version >> /out/result.txt
cat /out/result.txt
exit "$fails"
