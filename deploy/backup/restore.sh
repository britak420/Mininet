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
umask 077
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARCHIVE_TOOL="${SCRIPT_DIR}/state_archive.py"

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
command -v python3 >/dev/null 2>&1 || fail "python3 not found"
command -v flock >/dev/null 2>&1 || fail "flock not found"
[[ ! -L "${STATE_DIR}" ]] || fail "state directory must not be a symlink"
state_parent="$(cd "$(dirname "${STATE_DIR}")" && pwd -P)"
STATE_DIR="${state_parent}/$(basename "${STATE_DIR}")"
# Updated mini CLI processes hold a shared lease at this stable sibling path.
exec 9<> "${state_parent}/.mininet-maintenance-$(basename "${STATE_DIR}").lock"
flock --exclusive --nonblock 9 || fail "node state is in use; stop the sync service and other mini commands first"

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
trap 'rm -rf -- "${workdir}"' EXIT
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

state_name="$(basename "${STATE_DIR}")"
log "validating and extracting bounded regular-file archive"
python3 "${ARCHIVE_TOOL}" extract "${workdir}/state.tar.gz" "${workdir}/extracted" "${state_name}"
previous="${STATE_DIR}.restore-previous"
[[ ! -e "${previous}" && ! -L "${previous}" ]] \
    || fail "an earlier restore left ${previous}; reconcile it before restoring again"
python3 "${ARCHIVE_TOOL}" publish "${workdir}/extracted/${state_name}" "${STATE_DIR}" "${previous}"
[[ ! -e "${previous}" ]] || log "previous state retained at ${previous} for recovery"

# Ownership by NAME, not by the archived numeric uid: the mininet system
# user may well have a different uid on this machine.
if id -u mininet >/dev/null 2>&1; then
    chown -R mininet:mininet "${STATE_DIR}"
else
    log "the 'mininet' user does not exist yet; run the installer, then re-chown ${STATE_DIR}"
fi
chmod 0750 "${STATE_DIR}"
python3 "${ARCHIVE_TOOL}" validate-tree "${STATE_DIR}"

if [[ -f "${workdir}/extracted/appliance.conf" ]]; then
    install -d -m 0755 /etc/mininet
    if [[ -f /etc/mininet/appliance.conf ]]; then
        log "/etc/mininet/appliance.conf already exists; archived copy left at ${STATE_DIR}/appliance.conf.restored"
        install -m 0640 -o mininet -g mininet "${workdir}/extracted/appliance.conf" \
            "${STATE_DIR}/appliance.conf.restored"
    else
        install -m 0644 "${workdir}/extracted/appliance.conf" /etc/mininet/appliance.conf
    fi
fi

log "restored ${STATE_DIR}"
log "run deploy/verification/verify.sh, then start the node when you are satisfied"
