#!/usr/bin/env bash
# Prototype of signed-approvals: SSH commit signatures checked with
# `ssh-keygen -Y check-novalidate`, independent of the host's Git signing settings.
# Usage: proto.sh <empty work dir>
set -u
W=$1; rm -rf "$W"; mkdir -p "$W"; cd "$W" || exit 9
say() { printf '%s\n' "$*"; }
say "== tools"; ssh -V 2>&1; git --version; command -v ssh-keygen
ssh-keygen -q -t ed25519 -N "" -C owner-test -f owner
ssh-keygen -q -t ed25519 -N "" -C agent-test -f agent
git init -q repo; cd repo
git config user.name Tester; git config user.email t@example.invalid
git config --local --unset-all gpg.format 2>/dev/null
printf '{"write_offs":[{"fault":"a","approved":false}]}\n' > w.json
git add w.json; git commit -q -m "proposal, unsigned"
printf '{"write_offs":[{"fault":"a","approved":true}]}\n' > w.json
git add w.json; git -c gpg.format=ssh -c user.signingkey="$W/owner" commit -q -S -m "approval, signed by owner"
printf '{"write_offs":[{"fault":"a","approved":true,"reason":"x"}]}\n' > w.json
git add w.json; git -c gpg.format=ssh -c user.signingkey="$W/agent" commit -q -S -m "change, signed by agent key"
# split a commit object into payload (without the gpgsig header) and signature
split() {
  git cat-file commit "$1" > "$W/raw"
  awk 'h==0 && /^$/ {h=1} h==0 && /^gpgsig / {s=1; next} h==0 && s==1 && /^ / {next} {s=0; print}' "$W/raw" > "$W/payload"
  awk 'h==0 && /^$/ {h=1} h==0 && /^gpgsig / {s=1; sub(/^gpgsig /,""); print; next} h==0 && s==1 && /^ / {sub(/^ /,""); print; next} {s=0}' "$W/raw" > "$W/sig"
}
check() { ssh-keygen -Y check-novalidate -n git -s "$W/sig" < "$W/payload" 2>&1; echo "exit $?"; }
say "== fingerprints"; ssh-keygen -l -f "$W/owner.pub"; ssh-keygen -l -f "$W/agent.pub"
say "== P1 owner-signed commit (HEAD~1)"; split HEAD~1; check
say "== P2 agent-signed commit (HEAD): good signature, other key"; split HEAD; check
say "== P3 unsigned commit (HEAD~2): signature bytes"; split HEAD~2; wc -c < "$W/sig"
say "== A1 tampered payload of HEAD~1"; split HEAD~1; sed -i 's/approval, signed/approval, Signed/' "$W/payload"; check
say "== A2 wrong namespace"; split HEAD~1; ssh-keygen -Y check-novalidate -n file -s "$W/sig" < "$W/payload" 2>&1; echo "exit $?"
say "== C1 git's own verdict with an allowed-signers file naming the owner only"
printf 't@example.invalid namespaces="git" %s\n' "$(cat "$W/owner.pub")" > "$W/allowed"
git -c gpg.ssh.allowedSignersFile="$W/allowed" log --format='%h %G? %GF %s'
say "== C2 a signing that needs a passphrase cannot run without a terminal"
ssh-keygen -q -t ed25519 -N "secret-pass" -C pp -f "$W/pp"
printf 'x\n' > x; git add x
GIT_TERMINAL_PROMPT=0 SSH_ASKPASS= DISPLAY= git -c gpg.format=ssh -c user.signingkey="$W/pp" commit -q -S -m "needs passphrase" </dev/null 2>&1 | head -3; echo "exit ${PIPESTATUS[0]}"
