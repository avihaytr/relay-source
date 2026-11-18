#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
set -euo pipefail
cd "$(dirname "$0")/../.."
sdk="${ANDROID_HOME:-$HOME/Android/Sdk}"
build_tools="$sdk/build-tools/35.0.0"
platform="$sdk/platforms/android-35/android.jar"
python3 ../tools/check_licenses.py --sync-test-assets
mkdir -p build/device-tests/classes build/device-tests/dex
javac --release 8 -classpath "$platform:build/classes.jar" -d build/device-tests/classes tests/device/TrustStorageTest.java
jar cf build/device-tests/classes.jar -C build/device-tests/classes .
"$build_tools/d8" --lib "$platform" --classpath build/classes.jar --min-api 26 --output build/device-tests/dex build/device-tests/classes.jar
"$build_tools/aapt2" link -I "$platform" --manifest tests/device/AndroidManifest.xml -A build/device-tests/assets -o build/device-tests/unsigned.apk
python3 - <<'PY'
from zipfile import ZipFile
with ZipFile('build/device-tests/unsigned.apk', 'a') as archive:
    archive.write('build/device-tests/dex/classes.dex', 'classes.dex')
PY
"$build_tools/zipalign" -f 4 build/device-tests/unsigned.apk build/device-tests/aligned.apk
export RELAY_LOCAL_SIGNING_PASSWORD=relay-local-build
"$build_tools/apksigner" sign --ks relay.keystore --ks-pass env:RELAY_LOCAL_SIGNING_PASSWORD --out build/device-tests/tests.apk build/device-tests/aligned.apk
"$build_tools/apksigner" verify build/device-tests/tests.apk
python3 ../tools/check_licenses.py --test-apk build/device-tests/tests.apk
echo 'Built local instrumentation APK; production manifest is unchanged.'
