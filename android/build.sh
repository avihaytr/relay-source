#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
set -euo pipefail
cd "$(dirname "$0")"
sdk="${ANDROID_HOME:-$HOME/Android/Sdk}"
tools="$sdk/build-tools/35.0.0"
platform="$sdk/platforms/android-35/android.jar"
python3 ../tools/check_licenses.py --sync-android
mkdir -p build/classes build/dex build/res
"$tools/aapt2" compile --dir res -o build/res/resources.zip
javac --release 8 -classpath "$platform:lib/zxing-core.jar" -d build/classes src/app/relay/companion/*.java
jar cf build/classes.jar -C build/classes .
"$tools/d8" --lib "$platform" --min-api 26 --output build/dex build/classes.jar lib/zxing-core.jar
"$tools/aapt2" link -I "$platform" --auto-add-overlay -R build/res/resources.zip -A assets --manifest AndroidManifest.xml -o build/unsigned.apk
python3 - <<'PY'
from zipfile import ZipFile
with ZipFile('build/unsigned.apk', 'a') as z:
    z.write('build/dex/classes.dex', 'classes.dex')
PY
"$tools/zipalign" -f 4 build/unsigned.apk build/aligned.apk
if [ ! -f relay.keystore ]; then
  keytool -genkeypair -keystore relay.keystore -storepass relay-local-build -keypass relay-local-build -alias relay -keyalg RSA -validity 10000 -dname 'CN=Relay Local Build'
  chmod 600 relay.keystore
fi
"$tools/apksigner" sign --ks relay.keystore --ks-pass pass:relay-local-build --out relay.apk build/aligned.apk
"$tools/apksigner" verify relay.apk
python3 ../tools/check_licenses.py --apk relay.apk
echo 'Built android/relay.apk'
