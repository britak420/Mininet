import importlib.util
import io
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

MODULE = Path(__file__).with_name("state_archive.py")
spec = importlib.util.spec_from_file_location("state_archive", MODULE)
archive = importlib.util.module_from_spec(spec)
spec.loader.exec_module(archive)


class ArchiveTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.path = self.root / "input.tar.gz"

    def pack(self, entries):
        with tarfile.open(self.path, "w:gz") as out:
            parent = tarfile.TarInfo("mininet")
            parent.type = tarfile.DIRTYPE
            out.addfile(parent)
            for name, kind, data in entries:
                member = tarfile.TarInfo(name)
                member.type = kind
                member.linkname = "../../outside"
                member.size = len(data) if kind == tarfile.REGTYPE else 0
                out.addfile(member, io.BytesIO(data) if member.isfile() else None)

    def test_nested_special_members_rejected_before_extraction(self):
        for kind in (tarfile.SYMTYPE, tarfile.LNKTYPE, tarfile.CHRTYPE,
                     tarfile.BLKTYPE, tarfile.FIFOTYPE):
            with self.subTest(kind=kind):
                self.pack([("mininet/honest", tarfile.REGTYPE, b"ok"),
                           ("mininet/deep/attack", kind, b"")])
                destination = self.root / "extract"
                with self.assertRaises(ValueError):
                    archive.extract(self.path, destination, "mininet")
                self.assertFalse(destination.exists())

    def test_traversal_duplicate_and_parent_file_rejected(self):
        cases = [
            [("../outside", tarfile.REGTYPE, b"x")],
            [("/outside", tarfile.REGTYPE, b"x")],
            [("mininet/a", tarfile.REGTYPE, b"x")] * 2,
            [("mininet/a", tarfile.REGTYPE, b"x"),
             ("mininet/a/b", tarfile.REGTYPE, b"y")],
            [("mininet/../bad", tarfile.REGTYPE, b"x")],
            [("mininet/a\nb", tarfile.REGTYPE, b"x")],
        ]
        for case in cases:
            with self.subTest(case=case):
                self.pack(case)
                with self.assertRaises(ValueError):
                    archive.extract(self.path, self.root / "extract", "mininet")
                self.assertFalse((self.root / "extract").exists())

    def test_quota_failure_precedes_extraction(self):
        self.pack([("mininet/large", tarfile.REGTYPE, b"large")])
        with patch.object(archive, "MAX_TOTAL_BYTES", 1):
            with self.assertRaises(ValueError):
                archive.extract(self.path, self.root / "extract", "mininet")
        self.assertFalse((self.root / "extract").exists())

    @unittest.skipUnless(os.name == "posix", "appliance directory fsync requires POSIX")
    def test_valid_archive_flushes_and_recovers_exact_bytes(self):
        self.pack([("mininet/key", tarfile.REGTYPE, bytes(range(256))),
                   ("mininet/has..dots", tarfile.REGTYPE, b"valid name")])
        archive.extract(self.path, self.root / "extract", "mininet")
        self.assertEqual((self.root / "extract/mininet/key").read_bytes(), bytes(range(256)))
        self.assertEqual((self.root / "extract/mininet/key").stat().st_mode & 0o777, 0o600)

    @unittest.skipUnless(os.name == "posix", "appliance directory fsync requires POSIX")
    def test_real_restore_process_crash_preserves_old_or_new_state(self):
        for point in ("staged", "old_preserved", "new_published"):
            with self.subTest(point=point):
                root = self.root / point
                root.mkdir()
                state, staged, previous = (root / name for name in ("state", "stage", "previous"))
                state.mkdir(); staged.mkdir()
                (state / "key").write_bytes(b"old")
                (staged / "key").write_bytes(b"new")
                script = (
                    "import os,sys;sys.path.insert(0,sys.argv[1]);import state_archive;"
                    "state_archive.publish(*sys.argv[2:5],checkpoint=lambda p: os._exit(73) if p==sys.argv[5] else None)"
                )
                result = subprocess.run([sys.executable, "-c", script, str(MODULE.parent),
                                         str(staged), str(state), str(previous), point], check=False)
                self.assertEqual(result.returncode, 73)
                if point == "staged":
                    self.assertEqual((state / "key").read_bytes(), b"old")
                else:
                    self.assertEqual((previous / "key").read_bytes(), b"old")
                if point == "new_published":
                    self.assertEqual((state / "key").read_bytes(), b"new")


if __name__ == "__main__":
    unittest.main()
