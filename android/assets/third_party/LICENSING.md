# Relay licensing scope

The approved original source files listed below are licensed under standard MPL-2.0. [LICENSE](LICENSE) is the unmodified official license text. No Exhibit B incompatibility restriction is applied. Cargo metadata does not extend this grant to separate assets or third-party material.

## Source covered by MPL-2.0

- `Cargo.toml`
- `android/AndroidManifest.xml`
- `android/build.sh`
- `packaging/build-android-release.py`
- `packaging/make-source.py`
- `packaging/build-rpm.sh`

The RPM specification carries its own MPL-2.0 SPDX notice; desktop metadata is original application build metadata. Bundled upstream Cargo license texts and component declarations in `licenses/cargo/` and `licenses/CARGO_COMPONENTS.json` retain their separate upstream terms.
- `android/res/values/colors.xml`
- `android/src/app/relay/companion/DeviceTrust.java`
- `android/src/app/relay/companion/DirectionScreen.java`
- `android/src/app/relay/companion/DiscoveryScreen.java`
- `android/src/app/relay/companion/IdentityProof.java`
- `android/src/app/relay/companion/MainActivity.java`
- `android/src/app/relay/companion/PcViewerActivity.java`
- `android/src/app/relay/companion/RelayConnection.java`
- `android/src/app/relay/companion/ScanActivity.java`
- `android/src/app/relay/companion/SessionScreen.java`
- `android/src/app/relay/companion/SettingsScreen.java`
- `android/src/app/relay/companion/ShareService.java`
- `android/src/app/relay/companion/TrustStorage.java`
- `android/tests/IdentityProofTest.java`
- `android/tests/device/AndroidManifest.xml`
- `android/tests/device/TrustStorageTest.java`
- `android/tests/device/build.sh`
- `src/audio.rs`
- `src/companion.rs`
- `src/desktop.rs`
- `src/paths.rs`
- `src/main.rs`
- `src/mirror.rs`
- `src/pairing.rs`
- `src/pc_share.rs`
- `src/profile.rs`
- `src/trust.rs`
- `src/trust_tests.rs`
- `src/view.rs`
- `tools/check_licenses.py`
- `tools/desktop_capture.py`

The SPDX headers identify these approved files. This is not an independent authorship certificate or a grant of third-party rights. For `src/view.rs`, the grant includes the procedural illustration and header drawing code inside that file, not separate SVG/PNG or Android icon resources.

## Generated test data under MPL-2.0

- `tests/fixtures/java-p256-proof.txt`: generated public P-256 test vector from `IdentityProofTest --vector`. It contains public key, signature and synthetic transcript, not a private key. The explicit grant here replaces an inline header because readers consume exactly three lines.
- `tests/fixtures/keyframe.h264`: generated single synthetic red frame, included under MPL-2.0 to the extent rights exist. Preserve its embedded x264 encoder attribution. The encoder's source license does not by itself determine its output's license. See [fixture generation and hashes](tests/fixtures/README.md).

## Separate artwork copyright and product identity — adopted

The thirteen path/hash-listed artwork files in [the manifest](licenses/BRANDING_ASSET_MANIFEST.json) remain outside MPL. The owner adopted separate **MIT/Expat copyright permission, version 1.0, effective 2026-11-19**, in [ARTWORK_LICENSE.md](ARTWORK_LICENSE.md). That grant permits copying, modifying and redistributing the covered artwork, to the extent of rights held by the grantor; it does not relicense code, fonts or third-party files or invent a copyright holder. Actual copyright notices, if any, and the standard permission/warranty notice must be preserved. Provenance credits are not assertions of copyright ownership.

[BRANDING.md](BRANDING.md), **version 1.1, effective 2026-11-19**, is the separately adopted product-identity policy. It permits genuine unmodified builds and source-fork retention. Distinguishable unofficial forks, descriptive names such as Relay-Python, and independently created distinguishable branding remain permitted without false endorsement. The modified-product name/icon rule is a condition of applicable trademark permission, not an additional MIT/MPL/GPL copyright restriction. Lawful uses and actual rights limit enforcement. No exclusive copyright, registered trademark or third-party clearance is asserted.

The [archived v1.0 policy](licenses/BRANDING-v1.0.md) preserves the prior record; the current artwork copyright grant and product policy above supersede its restrictions to the extent stated here, without revoking rights already granted. Original-source and generated-fixture MPL grants remain unchanged. The Linux GPL-covered larger work uses the eligible MPL section 3.3 route while retaining MPL headers/source rights and satisfying GPLv3-or-later source obligations. Artwork copyright permission comes from the separate MIT grant, not automatically from MPL 3.3.

## Third-party material and binary distributions

Fonts, ZXing, Rust dependencies and native libraries retain their own terms. Preserve [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md), the [legal texts](licenses/README.md), and the original font/ZXing notices. Documentation and legal texts are not blanket-relicensed by the source grant.

The current Linux build links GPLv3-or-later FFmpeg and loads GPL x264 through GStreamer. MPL Section 3.3 supplies a conditional compatibility route, not a GPL exemption. Binary releases need their own enabled-dependency inventory, matching source delivery, notices and signing review. Source publication with external dependency instructions is distinct from distributing those compiled components. The experimental LGPL media variant is not production-ready and is not included in this source snapshot.

## Android notice packaging

`tools/check_licenses.py` checks source/data scope and preserved texts. Android builds copy this file, `LICENSE`, `BRANDING.md` and `licenses/RELAY-SOURCE.txt` to `assets/third_party/`, plus the adopted separate artwork copyright notice and ZXing's license/NOTICE in the main app. The build checks byte-identical legal entries. These source-tree links are intended for the checkout; the packaged source notice explains source availability for local builds. A public APK requires an accessible source location corresponding to its exact build; this preparation does not claim that requirement is satisfied.
