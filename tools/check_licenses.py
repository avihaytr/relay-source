# SPDX-License-Identifier: MPL-2.0
"""Check scoped source/data grants and byte-identical Android notice packaging."""
import argparse
import hashlib
import json
import re
from pathlib import Path
import shutil
import zipfile

ROOT = Path(__file__).resolve().parent.parent
MPL_SHA256 = "3f3d9e0024b1921b067d6f7f88deb4a60cbe7a78e76c64e3f1d7fc3b779b9d04"
ZXING_SHA256 = "8d8064c1636fdaef7189dd9055c7d59950a8940a12f2293956446ec3c109fd82"
COVERED = {
    "src/audio.rs", "src/companion.rs", "src/desktop.rs", "src/pairing.rs",
    "src/pc_share.rs", "src/profile.rs", "src/trust.rs", "src/trust_tests.rs",
    "src/main.rs", "src/mirror.rs", "src/view.rs",
    "tools/desktop_capture.py", "tools/check_licenses.py", "Cargo.toml",
    "android/build.sh", "android/AndroidManifest.xml",
    "android/tests/IdentityProofTest.java", "android/tests/device/build.sh",
    "android/tests/device/TrustStorageTest.java", "android/tests/device/AndroidManifest.xml",
    "android/res/values/colors.xml",
} | {"android/src/app/relay/companion/" + name + ".java" for name in (
    "DeviceTrust", "IdentityProof", "RelayConnection", "ShareService", "TrustStorage",
    "MainActivity", "PcViewerActivity", "ScanActivity", "SettingsScreen",
    "DiscoveryScreen", "DirectionScreen", "SessionScreen")}
PENDING = set()
MPL_DATA = {"tests/fixtures/java-p256-proof.txt", "tests/fixtures/keyframe.h264"}
COMMON_NOTICES = {
    "MPL-2.0.txt": "LICENSE",
    "BRANDING.md": "BRANDING.md",
    "LICENSING.md": "LICENSING.md",
    "RELAY-SOURCE.txt": "licenses/RELAY-SOURCE.txt",
}
ANDROID_NOTICES = dict(COMMON_NOTICES, **{
    "ZXING-LICENSE.txt": "android/lib/ZXING-LICENSE.txt",
    "ZXING-NOTICE.txt": "android/lib/ZXING-NOTICE.txt",
})


def check(condition, message):
    if not condition:
        raise ValueError(message)


def verify_source():
    check(hashlib.sha256((ROOT / "LICENSE").read_bytes()).hexdigest() == MPL_SHA256,
          "LICENSE differs from the official, unmodified MPL-2.0 copy")
    check(hashlib.sha256((ROOT / "android/lib/zxing-core.jar").read_bytes()).hexdigest() == ZXING_SHA256,
          "ZXing changed: review its version, licensing and notices")
    for name in COVERED:
        text = (ROOT / name).read_text()
        check("SPDX-License-Identifier: MPL-2.0" in "\n".join(text.splitlines()[:3]),
              "Missing approved SPDX header: " + name)
    for name in PENDING:
        check("SPDX-License-Identifier:" not in (ROOT / name).read_text(),
              "Pending UI source received a license without review: " + name)
    licensing = (ROOT / "LICENSING.md").read_text()
    for name in MPL_DATA:
        check(name in licensing and "Generated test data under MPL-2.0" in licensing,
              "MPL test-data scope is missing from LICENSING.md: " + name)
        if name == "tests/fixtures/keyframe.h264":
            check(hashlib.sha256((ROOT / name).read_bytes()).hexdigest() ==
                  "82d395953d01e24b07e7a2b698ecaf3beea09b869c47b7e22262fe9a9b34ede4",
                  "Synthetic H.264 fixture changed; review generation, attribution and scope")
            continue
        lines = (ROOT / name).read_text().splitlines()
        check(len(lines) == 3 and all(re.fullmatch(r"[0-9a-f]+", line) for line in lines),
              "MPL vector format changed; review its scope and parser: " + name)
    units = set()
    for directory, suffixes in (("src", {".rs"}), ("android/src", {".java"}),
                                ("android/tests", {".java", ".sh", ".xml"}),
                                ("tools", {".py"})):
        units.update(str(p.relative_to(ROOT)) for p in (ROOT / directory).rglob("*")
                     if p.is_file() and p.suffix in suffixes)
    units.update({"Cargo.toml", "android/build.sh", "android/AndroidManifest.xml"})
    units.update({"android/res/values/colors.xml"})
    check(units == COVERED | PENDING, "Source licensing scope changed; update LICENSING.md and this check")
    for base in (ROOT / "assets", ROOT / "android/res"):
        for path in base.rglob("*"):
            if path.suffix in {".svg", ".xml"}:
                check("SPDX-License-Identifier: MPL" not in path.read_text()
                      or str(path.relative_to(ROOT)) == "android/res/values/colors.xml",
                      "Uncleared artwork received MPL: " + str(path.relative_to(ROOT)))
    for record in json.loads((ROOT / "licenses/PROVENANCE.json").read_text())["files"]:
        check(hashlib.sha256((ROOT / record["path"]).read_bytes()).hexdigest() == record["sha256"],
              "Copied upstream legal text changed: " + record["path"])


def verify_branding():
    manifest = json.loads((ROOT / "licenses/BRANDING_ASSET_MANIFEST.json").read_text())
    check(manifest["mpl_scope"] == "excluded", "Branding MPL scope changed without review")
    check(manifest["status"] == "adopted-owner-approved",
          "Branding adoption status changed; review the approved policy")
    records = manifest["files"]
    expected = {"assets/relay-icon.svg", "assets/relay-icon.png",
                "android/res/drawable/ic_relay_vector.xml"}
    expected.update(str(p.relative_to(ROOT)) for p in (ROOT / "android/res").glob("mipmap-*/*.png"))
    check({r["path"] for r in records} == expected and len(records) == len(expected),
          "Branding asset inventory changed; review provenance and terms")
    for record in records:
        path = ROOT / record["path"]
        check(hashlib.sha256(path.read_bytes()).hexdigest() == record["sha256"],
              "Branding asset changed; review provenance: " + record["path"])
    policy = (ROOT / "BRANDING.md").read_text()
    check(manifest["policy_version"] == "1.0" and manifest["effective_date"] == "2026-11-18"
          and manifest["policy_file"] == "BRANDING.md", "Branding policy version/date changed")
    check(hashlib.sha256((ROOT / "BRANDING.md").read_bytes()).hexdigest() == manifest["policy_sha256"],
          "Branding policy changed; review approval and regenerate its recorded hash")
    check("ADOPTED — Version 1.0. Effective date: 2026-11-18" in policy,
          "Approved branding policy adoption record is missing")
    licensing = (ROOT / "LICENSING.md").read_text()
    check("BRANDING.md" in licensing and "policy adopted" in licensing,
          "Licensing scope must distinguish adopted branding terms from MPL-covered source")


def sync_notices(destination, notices):
    destination.mkdir(parents=True, exist_ok=True)
    for name, source in notices.items():
        shutil.copyfile(ROOT / source, destination / name)


def verify_apk(path, notices):
    with zipfile.ZipFile(path) as apk:
        for name, source in notices.items():
            check(apk.read("assets/third_party/" + name) == (ROOT / source).read_bytes(),
                  "APK notice is missing or stale: " + name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sync-android", action="store_true")
    parser.add_argument("--sync-test-assets", action="store_true")
    parser.add_argument("--apk", type=Path)
    parser.add_argument("--test-apk", type=Path)
    args = parser.parse_args()
    verify_source()
    verify_branding()
    if args.sync_android:
        sync_notices(ROOT / "android/assets/third_party", ANDROID_NOTICES)
    if args.sync_test_assets:
        sync_notices(ROOT / "android/build/device-tests/assets/third_party", COMMON_NOTICES)
    if args.apk:
        verify_apk(args.apk, ANDROID_NOTICES)
    if args.test_apk:
        verify_apk(args.test_apk, COMMON_NOTICES)
    print("PASS: licensing scope, official/third-party texts, branding hashes/policy v1.0 (adopted)" +
          (", APK notice contents" if args.apk or args.test_apk else ""))


if __name__ == "__main__":
    main()
