#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0
"""Build/sign a release APK; private keys/passwords are external inputs."""
import argparse
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--keystore", required=True, type=Path)
parser.add_argument("--password-env", default="RELAY_RELEASE_PASSWORD")
parser.add_argument("--alias", default="relay-release")
parser.add_argument("--old-keystore", type=Path)
parser.add_argument("--old-password-env", default="RELAY_OLD_PASSWORD")
parser.add_argument("--old-alias", default="relay")
parser.add_argument("--lineage", type=Path)
parser.add_argument("--out", required=True, type=Path)
args = parser.parse_args()
for variable in [args.password_env] + ([args.old_password_env] if args.old_keystore else []):
    if not os.environ.get(variable):
        raise SystemExit("Required signing-password environment variable is missing: " + variable)
if not args.keystore.is_file() or (args.old_keystore and not args.old_keystore.is_file()):
    raise SystemExit("Required external keystore is missing")
if args.old_keystore and not args.lineage:
    raise SystemExit("Key rotation requires an explicit lineage output path")
sdk = Path(os.environ.get("ANDROID_HOME", str(Path.home() / "Android/Sdk")))
signer = sdk / "build-tools/35.0.0/apksigner"
build_env = os.environ.copy()
for variable in [args.password_env, args.old_password_env]:
    build_env.pop(variable, None)
subprocess.run(["bash", "android/build.sh", "--unsigned"], cwd=ROOT, env=build_env, check=True)
args.out.parent.mkdir(parents=True, exist_ok=True)
new = ["--ks", str(args.keystore.resolve()), "--ks-key-alias", args.alias, "--ks-pass", "env:" + args.password_env]
old = []
if args.old_keystore:
    old = ["--ks", str(args.old_keystore.resolve()), "--ks-key-alias", args.old_alias, "--ks-pass", "env:" + args.old_password_env]
    args.lineage.parent.mkdir(parents=True, exist_ok=True)
    if not args.lineage.exists():
        subprocess.run([str(signer), "rotate", "--out", str(args.lineage), "--old-signer", *old, "--new-signer", *new], check=True)
rotation = ["--lineage", str(args.lineage), "--rotation-min-sdk-version", "28"] if args.old_keystore else []
signers = [*old, "--next-signer", *new] if args.old_keystore else new
subprocess.run([str(signer), "sign", *signers, *rotation, "--v4-signing-enabled", "false", "--out", str(args.out), str(ROOT / "android/build/aligned.apk")], check=True)
subprocess.run([str(signer), "verify", "--verbose", "--print-certs", str(args.out)], check=True)
subprocess.run(["python3", "tools/check_licenses.py", "--apk", str(args.out)], cwd=ROOT, check=True)
