# Release signing

The scheme is the one used by mmhmem, with a separate key. Two signatures made
with the same Ed25519 key protect each release:

| What | Signature |
| --- | --- |
| each archive `lesscode-core-<target>.tar.gz` / `.zip` | zipsign signature embedded in the archive, context = the published file name |
| `SHA256SUMS` (computed after signing, over every published file) | detached `SHA256SUMS.sig`, zipsign format (`zipsign sign separate`), context `SHA256SUMS` |

A signed archive is still an ordinary `.tar.gz` or `.zip`; `tar` and
`Expand-Archive` read it as before. Renaming an archive breaks its signature,
because the file name is the signing context. Ed25519 is deterministic, so a
retried publish produces identical bytes.

The private key lives only in the GitHub `release` environment. The public
keys are listed in [scripts/release-public-keys.txt](../scripts/release-public-keys.txt).
The publisher refuses to publish without a key, with another zipsign version,
or with a key that is not in that list. The list is empty until the owner adds
a key, so no release can be published before the steps below are done.

The pinned signing tool is `zipsign` 0.2.2. Never paste a real key into an
issue, a commit, a chat or a log.

Release 1.0.1 was published before signing existed. Its archives carry SHA-256
files but no signatures.

## Verify a download

The install scripts check each archive against its `.sha256` file. That
detects a damaged or truncated download; it does not prove who made the file,
because the digest comes from the same place as the archive. To check the
signature, take the public key from a clone of this repository that you trust
and use zipsign:

```sh
cargo install zipsign --version =0.2.2 --locked
# 64 hex characters from scripts/release-public-keys.txt, as a 32-byte file:
grep -v '^#' scripts/release-public-keys.txt | awk 'NF {print $1; exit}' | xxd -r -p > release.pub

zipsign verify tar --context lesscode-core-x86_64-unknown-linux-gnu.tar.gz \
  lesscode-core-x86_64-unknown-linux-gnu.tar.gz release.pub
zipsign verify separate --context SHA256SUMS SHA256SUMS SHA256SUMS.sig release.pub
```

Use `zipsign verify zip` for the Windows archive. `SHA256SUMS` also covers the
install scripts, so a verified `SHA256SUMS` lets you check `install.sh` and
`install.ps1` with `sha256sum -c` before running them.

Each archive also has a GitHub build provenance attestation:

```sh
gh attestation verify lesscode-core-x86_64-unknown-linux-gnu.tar.gz --repo norandom/lesscode
```

## Owner steps

### Quick way: one script for steps 1 to 3

From the repository root on your own machine:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\setup-release-signing.ps1
```

It installs zipsign, creates the key in `~\lesscode-release-key` (never
replacing an existing one), creates the `release` environment with you as
reviewer, stores the secret without printing it, and adds the public key to
`scripts/release-public-keys.txt`. Running it again is safe. Then commit that
file and push it to main. The sections below are the same steps by hand.

### 1. Generate the key pair (once, on your own machine)

```powershell
cargo install zipsign --version =0.2.2 --locked
zipsign --version        # must print: zipsign 0.2.2
New-Item -ItemType Directory -Force $HOME\lesscode-release-key | Out-Null
zipsign gen-key $HOME\lesscode-release-key\release.key $HOME\lesscode-release-key\release.pub
```

`release.key` is the 64-byte private key file. `release.pub` is the 32-byte
public key. Keep `release.key` off shared drives; back it up in a password
manager or offline. Do not reuse the mmhmem key.

### 2. Store the private key as the `release` environment secret

```powershell
$repo = 'norandom/lesscode'
$reviewer = (gh api user --jq .id)
'{"reviewers":[{"type":"User","id":' + $reviewer + '}],"deployment_branch_policy":null}' |
  gh api --method PUT "repos/$repo/environments/release" --input -

[Convert]::ToBase64String([IO.File]::ReadAllBytes("$HOME\lesscode-release-key\release.key")) |
  gh secret set LESSCODE_RELEASE_SIGNING_KEY --env release --repo $repo

gh secret list --env release --repo $repo   # names only
```

Do not add this secret as a repository or organization secret: only the
`release` environment, behind approval, may hold it.

### 3. Publish the public key

```powershell
$b = [IO.File]::ReadAllBytes("$HOME\lesscode-release-key\release.pub")
if ($b.Length -ne 32) { throw 'unexpected public key size' }
($b | ForEach-Object { $_.ToString('x2') }) -join ''
```

Add the printed line to `scripts/release-public-keys.txt`, commit and push.
The key list must be on the commit you tag.

### 4. Publishing

Push a `vMAJOR.MINOR.PATCH` tag on a commit with a green `Native core
correctness` run. The five build jobs run without secrets. The `publish` job
then waits for your approval of the `release` environment, signs every
archive, writes the `.sha256` files and `SHA256SUMS`, signs `SHA256SUMS`,
attests build provenance for the archives, uploads everything to a draft and
publishes the draft as the last step. See
[CI validation](ci-validation.md#release-binaries).

### 5. Rotate keys

1. Generate a new pair as in step 1 (a new directory).
2. Add the new public key to `scripts/release-public-keys.txt` next to the old
   one and push.
3. Replace the secret (`gh secret set LESSCODE_RELEASE_SIGNING_KEY --env release`
   with the new base64 value). Releases are now signed with the new key.
4. Keep the old key in the list for as long as releases signed with it should
   stay verifiable. Mark it with a comment and the date it stopped signing.
5. If the old key may be compromised, remove it from the list at once and say
   so in the release notes; releases signed with it can no longer be trusted.
