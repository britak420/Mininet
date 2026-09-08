#!/usr/bin/env bash
# Mininet Node Appliance — node state restore.
#
# # The thing to understand before running this
#
# Restoring node state is not like restoring a database. The archive
# contains a `did:mini` identity, and an identity is meant to exist in one
# place. Restoring it onto a second machine while the original still runs
# does not give you a replica — it gives you two machines signing as one
# identity, which is **equivocation**: the same failure `mini-consensus`
# treats as attributable misbehavior, and the same shape as the
# warehouse-with-many-identities problem `mini-storage-fraud` exists to
# detect, only inverted.
#
# So this script refuses to run over an existing state directory unless you
# say so explicitly, and it says out loud what you are doing when you do.
# It cannot check whether the original node is still running — nothing
# local can — which is exactly why the confirmation is manual.
#
# Usage:
#   deploy/backup/restore.sh /path/to/mininet-node-<stamp>.tar.gz.gpg
#   deploy/backup/restore.sh <archive> --force        # overwrite existing state
#   MININET_BACKUP_PASSPHRASE=... deploy/backup/restore.sh <archive> --batch

set -euo pipefail

STATE_DIR="${MININET_STATE_DIR:-/var/lib/mininet}"

BATCH=0
FORCE=0
ARCHIVE=""
for arg in "$@"; do
    case "${arg}" in
        --batch) BATCH=1 ;;
        --force) FORCE=1 ;;
        -*) printf 'unknown argument: %s\n' "${arg}" >&2; exit 1 ;;
        *) ARCHIVE="${arg}" ;;
    esac
done

log() { printf '[mininet-restore] %s\n' "$*"; }
fail() { printf '[mininet-restore] ERROR: %s\n' "$*" >&2; exit 1; }

[[ -n "${ARCHIVE}" ]] || fail "usage: restore.sh <archive.tar.gz.gpg> [--force] [--batch]"
[[ -r "${ARCHIVE}" ]] || fail "cannot read ${ARCHIVE}"
[[ "${EUID}" -eq 0 ]] || fail "must run as root (writes ${STATE_DIR} and /etc/mininet)"

command -v gpg >/dev/null 2>&1 || fail "gpg not found (install the gnupg package)"

# Integrity first, so a truncated download is distinguishable from a wrong
# passphrase. Without this the operator sees "decryption failed" for both.
manifest="${ARCHIVE%.tar.gz.gpg}.manifest"
if [[ -r "${manifest}" ]] && command -v sha256sum >/dev/null 2>&1; then
    expected="$(cat "${manifest}")"
    actual="$(sha256sum "${ARCHIVE}" | awk '{print $1}')"
    if [[ "${expected}" != "${actual}" ]]; then
        fail "archive digest does not match its manifest — the file is corrupt or truncated, not merely locked"
    fi
    log "archive digest matches its manifest"
else
    log "no manifest beside the archive; skipping the integrity pre-check"
fi

if [[ -d "${STATE_DIR}" ]] && [[ -n "$(ls -A "${STATE_DIR}" 2>/dev/null)" ]]; then
    if [[ "${FORCE}" -ne 1 ]]; then
        fail "${STATE_DIR} already contains state. Restoring over it would discard this node's current identity, which cannot be recovered. Pass --force if that is genuinely what you want."
    fi
    log "WARNING: overwriting existing state in ${STATE_DIR} (--force)"
    log "         the identity currently on this machine will be unrecoverable"
fi

log ""
log "Before restoring, confirm the machine this archive came from is NOT still"
log "running as this identity. Two machines signing as one identity is"
log "equivocation, and the network cannot tell it from an attack."
log ""

if [[ "${BATCH}" -eq 1 ]]; then
    [[ -n "${MININET_BACKUP_PASSPHRASE:-}" ]] \
        || fail "--batch requires MININET_BACKUP_PASSPHRASE in the environment"
else
    read -r -p "[mininet-restore] Type 'restore' to continue: " confirm
    [[ "${confirm}" == "restore" ]] || fail "aborted"
fi

# F-16: staged on the same filesystem as STATE_DIR's own parent (not the
# system default temp filesystem, e.g. tmpfs at /tmp, which is very often
# a different mount) so the final `mv` operations below are real,
# same-filesystem atomic renames rather than a cross-filesystem copy that
# a crash or power loss partway through could leave incomplete.
install -d -m 0750 "$(dirname "${STATE_DIR}")"
workdir="$(mktemp -d "$(dirname "${STATE_DIR}")/.mininet-restore.XXXXXX")"
# shellcheck disable=SC2064 # expand workdir now, not at trap time
trap "rm -rf -- '${workdir}'" EXIT
chmod 0700 "${workdir}"

log "decrypting"
if [[ "${BATCH}" -eq 1 ]]; then
    # F-16: --passphrase-fd, not --passphrase -- see backup.sh's own
    # comment on this. The archive is read as a plain positional argument
    # (its path, not a secret) and is unaffected.
    gpg --batch --yes --quiet --decrypt \
        --passphrase-fd 3 "${ARCHIVE}" \
        > "${workdir}/state.tar.gz" \
        3< <(printf '%s' "${MININET_BACKUP_PASSPHRASE}") \
        || fail "decryption failed (wrong passphrase, or an archive this key does not open)"
else
    gpg --quiet --decrypt "${ARCHIVE}" > "${workdir}/state.tar.gz" \
        || fail "decryption failed (wrong passphrase, or an archive this key does not open)"
fi

# F-16: reject any archive member that could write outside the staging
# directory before ever extracting one byte -- an absolute path, or a
# `..` path-traversal component anywhere in the path, rather than trusting
# the installed tar's own default protections (which vary by version and
# by whether `-P`/`--absolute-names` was ever set in some other context).
log "validating archive member paths"
while IFS= read -r member; do
    case "${member}" in
        /*) fail "archive contains an absolute path, refusing to extract: ${member}" ;;
        *..*) fail "archive contains a path-traversal component, refusing to extract: ${member}" ;;
    esac
done < <(tar -tzf "${workdir}/state.tar.gz")

log "extracting"
tar -xzf "${workdir}/state.tar.gz" -C "${workdir}"

state_name="$(basename "${STATE_DIR}")"
[[ -d "${workdir}/${state_name}" ]] \
    || fail "archive does not contain a '${state_name}' directory — is this a Mininet node backup?"
# The extracted directory must be real, not a symlink escaping the
# staging area -- a malicious archive could otherwise plant a symlink
# named exactly `${state_name}` pointing anywhere on the filesystem.
[[ ! -L "${workdir}/${state_name}" ]] \
    || fail "archive's '${state_name}' entry is a symlink, refusing to treat it as extracted state"

# F-16: never leave neither a usable old state nor a complete new one.
# `workdir` (via mktemp -d below) is created on the same filesystem as
# STATE_DIR's own parent specifically so this whole sequence is a chain
# of same-filesystem renames (atomic at the OS level), not a delete
# followed by a cross-filesystem copy: the previous state is moved aside
# -- not deleted -- before the new one is moved into place, and only
# removed once the new state already occupies STATE_DIR. An interruption
# at any point before the final `rm -rf` still leaves either the fully
# intact previous state (at its `.previous` path) or the fully intact new
# state (already at STATE_DIR) recoverable; it can never lose both.
previous="${STATE_DIR}.previous.$$"
if [[ -e "${STATE_DIR}" ]]; then
    mv -- "${STATE_DIR}" "${previous}"
fi
mv -- "${workdir}/${state_name}" "${STATE_DIR}"
if [[ -e "${previous}" ]]; then
    rm -rf -- "${previous}"
fi

# Ownership by NAME, not by the archived numeric uid: the mininet system
# user may well have a different uid on this machine.
if id -u mininet >/dev/null 2>&1; then
    chown -R mininet:mininet "${STATE_DIR}"
else
    log "the 'mininet' user does not exist yet; run the installer, then re-chown ${STATE_DIR}"
fi
chmod 0750 "${STATE_DIR}"

if [[ -f "${workdir}/appliance.conf" ]]; then
    install -d -m 0755 /etc/mininet
    if [[ -f /etc/mininet/appliance.conf ]]; then
        log "/etc/mininet/appliance.conf already exists; archived copy left at ${STATE_DIR}/appliance.conf.restored"
        install -m 0640 -o mininet -g mininet "${workdir}/appliance.conf" \
            "${STATE_DIR}/appliance.conf.restored"
    else
        install -m 0644 "${workdir}/appliance.conf" /etc/mininet/appliance.conf
    fi
fi

log "restored ${STATE_DIR}"
log "run deploy/verification/verify.sh, then start the node when you are satisfied"
