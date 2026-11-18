# Relay licensing scope

The approved original source files listed below are licensed under standard MPL-2.0. [LICENSE](LICENSE) is the unmodified official license text. No Exhibit B incompatibility restriction is applied. Cargo metadata does not extend this grant to separate assets or third-party material.

## Source covered by MPL-2.0

- `Cargo.toml`
- `android/AndroidManifest.xml`
- `android/build.sh`
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

## Separate branding: excluded from MPL; policy adopted

`assets/relay-icon.svg`, `assets/relay-icon.png`, `android/res/drawable/ic_relay_vector.xml` and the ten launcher PNGs listed in [the branding manifest](licenses/BRANDING_ASSET_MANIFEST.json) remain excluded from MPL. [BRANDING.md](BRANDING.md) contains the separately adopted branding policy: **version 1.0, effective 2026-11-18**, explicitly approved by the owner. This adoption does not assert exclusive copyright, registered trademarks or independently verified third-party clearance; independent rights in the earlier design reference remain unverified.

The policy permits branding with genuine unmodified Relay builds and retention of original assets in source forks for completeness, attribution and reproducibility without implying endorsement. Independently distributed modified application builds require a distinguishable product name and icon unless separately authorized. Descriptive names such as “Relay-Python” are permitted with distinct visual branding and clear independent-project identification. Independently created branding may draw on general Relay design concepts if sufficiently distinguishable and without implying endorsement or official status. These branding rules do not restrict MPL-covered code rights.

## Third-party material and binary distributions

Fonts, ZXing, Rust dependencies and native libraries retain their own terms. Preserve [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md), the [legal texts](licenses/README.md), and the original font/ZXing notices. Documentation and legal texts are not blanket-relicensed by the source grant.

The current Linux build links GPLv3-or-later FFmpeg and loads GPL x264 through GStreamer. MPL Section 3.3 supplies a conditional compatibility route, not a GPL exemption. Binary releases need their own enabled-dependency inventory, matching source delivery, notices and signing review. Source publication with external dependency instructions is distinct from distributing those compiled components. The experimental LGPL media variant is not production-ready and is not included in this source snapshot.

## Android notice packaging

`tools/check_licenses.py` checks source/data scope and preserved texts. Android builds copy this file, `LICENSE`, `BRANDING.md` and `licenses/RELAY-SOURCE.txt` to `assets/third_party/`, plus ZXing's license/NOTICE in the main app. The build checks byte-identical legal entries. These source-tree links are intended for the checkout; the packaged source notice explains source availability for local builds. A public APK requires an accessible source location corresponding to its exact build; this preparation does not claim that requirement is satisfied.
