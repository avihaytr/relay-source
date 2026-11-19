#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Create a deterministic, history-free source archive without private/generated files."""
from pathlib import Path
import argparse
import tarfile
import os
ROOT = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--out", required=True, type=Path)
args = parser.parse_args()
allowed = {"ARTWORK_LICENSE.md", ".gitignore", "BRANDING.md", "CONTRIBUTING.md", "Cargo.lock", "Cargo.toml", "LICENSE", "LICENSING.md", "README.md", "SECURITY.md", "THIRD_PARTY_NOTICES.md"}
roots = {"src", "tools", "tests", "android", "assets", "licenses", "packaging", "screenshots"}
files = []
for path in ROOT.rglob("*"):
    if not path.is_file(): continue
    rel = path.relative_to(ROOT)
    if path.is_symlink(): raise SystemExit("Symlinks are not permitted in the source candidate")
    if len(rel.parts) == 1 and str(rel) not in allowed: continue
    if len(rel.parts) > 1 and rel.parts[0] not in roots: continue
    if any(p in {"build", "__pycache__", ".git", "docs", "vendor", "target"} for p in rel.parts): continue
    if path.suffix in {".apk", ".idsig", ".p12", ".jks", ".keystore", ".pem", ".key", ".log", ".pyc"}: continue
    files.append(path)
args.out.parent.mkdir(parents=True, exist_ok=True)
with tarfile.open(args.out, "w:xz", format=tarfile.PAX_FORMAT) as archive:
    for path in sorted(files):
        info = archive.gettarinfo(str(path), "relay-0.1.0-alpha.1/" + path.relative_to(ROOT).as_posix())
        info.uid = info.gid = 0
        info.uname = info.gname = ""
        info.mtime = int(os.environ.get("SOURCE_DATE_EPOCH", "1795039390"))
        info.pax_headers = {}
        with path.open("rb") as source: archive.addfile(info, source)
print("Archived", len(files), "source files; no private history or signing keys")
