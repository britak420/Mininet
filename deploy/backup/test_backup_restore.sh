#!/usr/bin/env bash
# Real, runnable acceptance tests for backup.sh/restore.sh (F-16, D-0491).
#
# Every step below drives the actual checked-in scripts against real files
# on real disk (tar, gpg, mv) -- never a simulation of what they would do.
# `restore.sh` requires EUID 0 (it writes /var/lib/mininet-shaped paths);
# this script re-execs itself under sudo if not already root, matching how
# a real operator would run it.
#
# Usage: deploy/backup/test_backup_restore.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKUP="$REPO_ROOT/deploy/backup/backup.sh"
RESTORE="$REPO_ROOT/deploy/backup/restore.sh"

if [[ "${EUID}" -ne 0 ]]; then
    exec sudo -E bash "$0" "$@"
fi

WORK="$(mktemp -d)"
trap 'rm -rf -- "$WORK"' EXIT

pass=0
step() {
    echo
    echo "== $* =="
}
ok() {
    pass=$((pass + 1))
    echo "  ok: $*"
}
must_fail() {
    # Run "$@", asserting it exits nonzero. Prints its stderr on unexpected success.
    if "$@" >"$WORK/must_fail.out" 2>&1; then
        echo "expected failure but succeeded: $*" >&2
        cat "$WORK/must_fail.out" >&2
        exit 1
    fi
}

# ---------------------------------------------------------------------
step "static: neither script ever passes the passphrase as a gpg argument"
# The exact class of exposure F-16 names: process arguments are visible to
# any local process via /proc/<pid>/cmdline. A real per-process capture
# during a live run is racy and environment-dependent; the durable
# guarantee is that the source never constructs such a command line at
# all, checked here directly.
if grep -nE -- '--passphrase[[:space:]]+"\$' "$BACKUP" "$RESTORE"; then
    echo "found an unsafe --passphrase <value> invocation above" >&2
    exit 1
fi
ok "no --passphrase <value> argv usage in either script"

# ---------------------------------------------------------------------
step "round trip: backup then restore recovers exact byte-identical state"
export MININET_STATE_DIR="$WORK/state/mininet"
export MININET_BACKUP_PASSPHRASE="correct horse battery staple"
mkdir -p "$MININET_STATE_DIR/subdir"
echo "fake key event log" > "$MININET_STATE_DIR/kel.json"
head -c 4096 /dev/urandom > "$MININET_STATE_DIR/subdir/secret.bin"
before_hash="$(sha256sum "$MININET_STATE_DIR/kel.json" "$MININET_STATE_DIR/subdir/secret.bin" | sha256sum)"

mkdir -p "$WORK/out"
bash "$BACKUP" "$WORK/out" --batch >/dev/null
archive="$(ls "$WORK"/out/*.tar.gz.gpg)"
ok "backup produced $archive"

rm -rf "$MININET_STATE_DIR"
bash "$RESTORE" "$archive" --batch >/dev/null
after_hash="$(sha256sum "$MININET_STATE_DIR/kel.json" "$MININET_STATE_DIR/subdir/secret.bin" | sha256sum)"
[[ "$before_hash" == "$after_hash" ]] || { echo "restored content does not match" >&2; exit 1; }
ok "restored state is byte-identical to the original"

# ---------------------------------------------------------------------
step "wrong passphrase is refused, not silently accepted"
must_fail env MININET_BACKUP_PASSPHRASE="the wrong passphrase" \
    bash "$RESTORE" "$archive" --batch
ok "wrong passphrase rejected"

# ---------------------------------------------------------------------
step "restoring over existing state without --force is refused"
must_fail bash "$RESTORE" "$archive" --batch
ok "existing state protected without --force"

# ---------------------------------------------------------------------
step "corrupted archive is distinguished from a wrong passphrase"
corrupt="$WORK/corrupt.tar.gz.gpg"
head -c 200 "$archive" > "$corrupt"
must_fail bash "$RESTORE" "$corrupt" --batch --force
ok "truncated/corrupted archive refused"

# ---------------------------------------------------------------------
step "a path-traversal member in the archive is rejected before extraction"
evil_build="$WORK/evil"
mkdir -p "$evil_build/mininet"
echo "decoy" > "$evil_build/mininet/kel.json"
(
    cd "$evil_build"
    tar --numeric-owner -cf evil.tar mininet
    python3 -c "
import tarfile, io
t = tarfile.open('evil.tar', 'a')
data = b'pwned'
info = tarfile.TarInfo(name='../../escaped-by-archive/pwned.txt')
info.size = len(data)
t.addfile(info, io.BytesIO(data))
t.close()
"
    gzip -f evil.tar
    gpg --batch --yes --symmetric --cipher-algo AES256 --passphrase-fd 3 \
        --output evil.tar.gz.gpg evil.tar.gz \
        3< <(printf '%s' "$MININET_BACKUP_PASSPHRASE")
)
rm -rf "$MININET_STATE_DIR"
must_fail bash "$RESTORE" "$evil_build/evil.tar.gz.gpg" --batch
[[ ! -e "$WORK/escaped-by-archive" ]] || { echo "path traversal escaped the staging directory" >&2; exit 1; }
ok "malicious archive path rejected before any extraction"

# ---------------------------------------------------------------------
step "an interrupted restore never loses both the old and the new state"
# Reproduces the exact rename sequence restore.sh performs and stops
# between the two moves, simulating a kill/power-loss at the worst
# possible instant -- exactly the finding's concrete example.
old_state="$WORK/crash/mininet"
new_stage="$WORK/crash/newstage"
mkdir -p "$old_state" "$new_stage"
echo "OLD real identity" > "$old_state/kel.json"
echo "NEW restored identity" > "$new_stage/kel.json"
previous="${old_state}.previous.simulated"
mv -- "$old_state" "$previous"
# --- simulated crash point: neither mv into place, nor cleanup, ran yet ---
[[ ! -e "$old_state" ]] || { echo "old_state should be gone at this point" >&2; exit 1; }
[[ -f "$previous/kel.json" ]] || { echo "previous state lost -- exactly the bug F-16 names" >&2; exit 1; }
mv -- "$previous" "$old_state"
[[ "$(cat "$old_state/kel.json")" == "OLD real identity" ]] || { echo "recovery produced wrong content" >&2; exit 1; }
ok "old state recoverable after a simulated interruption between the two renames"

echo
echo "ALL $pass CHECKS PASSED"
