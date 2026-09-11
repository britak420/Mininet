#!/usr/bin/env python3
"""Bounded, regular-file-only appliance archives and durable restore publication.

The shell entry points hold the node's exclusive maintenance lease throughout.
Only updated cooperating CLI writers participate; this is not a filesystem
snapshot against arbitrary privileged writes or protection against disk rollback.
"""
import argparse
import os
from pathlib import Path, PurePosixPath
import shutil
import tarfile

MAX_MEMBERS = 100_000
MAX_TOTAL_BYTES = 64 * 1024**3
MAX_FILE_BYTES = 16 * 1024**3


def sync_directory(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def sync_tree(path):
    for base, dirs, files in os.walk(path, followlinks=False):
        for name in dirs + files:
            item = Path(base) / name
            if item.is_symlink():
                raise ValueError("state contains a symlink")
        for name in files:
            item = Path(base) / name
            if not item.is_file():
                raise ValueError("state contains a special file")
            with item.open("rb") as stream:
                os.fsync(stream.fileno())
    for base, _, _ in os.walk(path, topdown=False):
        sync_directory(base)


def extract(archive, destination, state_name):
    """Validate every member before creating any file; never call extractall."""
    destination = Path(destination)
    seen = {}
    members = []
    total = 0
    with tarfile.open(archive, "r:gz") as source:
        for member in source:
            raw = member.name.rstrip("/")
            path = PurePosixPath(raw)
            parts = raw.split("/")
            if (not raw or path.is_absolute() or "\\" in raw or ":" in raw
                    or any(part in ("", ".", "..") for part in parts)
                    or any(ord(char) < 32 for char in raw)
                    or len(raw.encode("utf-8")) > 4096):
                raise ValueError("unsafe archive path")
            if parts[0] != state_name and raw != "appliance.conf":
                raise ValueError("unexpected archive root")
            if raw == state_name and not member.isdir():
                raise ValueError("state root must be a directory")
            if raw == "appliance.conf" and not member.isfile():
                raise ValueError("configuration must be a regular file")
            if not (member.isdir() or member.isfile()) or member.sparse is not None:
                raise ValueError("links, sparse files and special archive members are forbidden")
            if raw in seen:
                raise ValueError("duplicate archive member")
            if member.size < 0 or member.size > MAX_FILE_BYTES:
                raise ValueError("archive member exceeds byte limit")
            total += member.size
            if total > MAX_TOTAL_BYTES or len(members) >= MAX_MEMBERS:
                raise ValueError("archive exceeds quota")
            seen[raw] = member.isdir()
            members.append((member, parts))
        if seen.get(state_name) is not True:
            raise ValueError("archive has no state directory")
        for _, parts in members:
            for index in range(1, len(parts)):
                if seen.get("/".join(parts[:index])) is False:
                    raise ValueError("regular file used as a parent directory")
        destination.mkdir(mode=0o700, parents=True, exist_ok=True)
        if any(destination.iterdir()):
            raise ValueError("extraction destination must be empty")
        for member, parts in members:
            target = destination.joinpath(*parts)
            if member.isdir():
                target.mkdir(mode=0o700, parents=True, exist_ok=True)
            else:
                target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
                with source.extractfile(member) as reader, target.open("xb") as writer:
                    shutil.copyfileobj(reader, writer, length=1024 * 1024)
                    writer.flush()
                    os.fchmod(writer.fileno(), 0o600)
                    os.fsync(writer.fileno())
                if target.stat().st_size != member.size:
                    raise ValueError("truncated archive member")
        sync_tree(destination)


def publish(staged, state, previous, checkpoint=lambda _: None):
    """Durable two-rename publication. Retain previous state for recovery.

    An exception after moving old state restores its name when possible.
    A killed process leaves the known previous path; never delete that copy.
    """
    staged, state, previous = map(Path, (staged, state, previous))
    if state.is_symlink() or previous.exists() or previous.is_symlink():
        raise ValueError("unsafe or unresolved previous restore state")
    if state.parent.resolve() != previous.parent.resolve():
        raise ValueError("previous state must share the state parent")
    sync_tree(staged)
    checkpoint("staged")
    moved_old = False
    try:
        if state.exists():
            os.rename(state, previous)
            moved_old = True
            sync_directory(state.parent)
        checkpoint("old_preserved")
        os.rename(staged, state)
        sync_directory(state.parent)
        checkpoint("new_published")
    except BaseException:
        if moved_old and not state.exists():
            os.rename(previous, state)
            sync_directory(state.parent)
        raise


def sync_file(path):
    with Path(path).open("rb") as stream:
        os.fsync(stream.fileno())
    sync_directory(Path(path).parent)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    unpack = commands.add_parser("extract")
    unpack.add_argument("archive")
    unpack.add_argument("destination")
    unpack.add_argument("state_name")
    commit = commands.add_parser("publish")
    commit.add_argument("staged")
    commit.add_argument("state")
    commit.add_argument("previous")
    flush = commands.add_parser("sync-file")
    flush.add_argument("path")
    tree = commands.add_parser("validate-tree")
    tree.add_argument("path")
    args = parser.parse_args()
    if args.command == "extract":
        extract(args.archive, args.destination, args.state_name)
    elif args.command == "publish":
        publish(args.staged, args.state, args.previous)
    elif args.command == "validate-tree":
        sync_tree(args.path)
    else:
        sync_file(args.path)


if __name__ == "__main__":
    main()
