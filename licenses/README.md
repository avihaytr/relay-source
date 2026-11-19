# Preserved third-party legal texts

These are **unmodified copies of upstream notices**, not Relay MPL-covered source. `PROVENANCE.json` records their originating locked crate/file or official URL and SHA-256. Registry archive checksums for the exact crate versions are in `Cargo.lock`.

- `epaint_default_fonts-0.33.3/fonts/`: complete packaged Hack, OFL, Ubuntu Font Licence and emoji-icon-font license texts. `FONT-COPYRIGHTS.txt` records copyright/version metadata from the embedded TTFs and the upstream emoji MIT notice; it does not replace those texts. The separately embedded Noto Sans notice remains at `assets/FONT-LICENSE.txt`.
- `ring-0.17.14/`: the root license guide, BoringSSL Apache-2.0 text, ISC notice and nested once_cell MIT/Apache texts. Individual upstream file notices also remain applicable; these copies do not certify a complete binary closure audit.
- ZXing's existing Apache license remains `android/lib/ZXING-LICENSE.txt`; its full, unmodified 3.5.3 upstream NOTICE is `android/lib/ZXING-NOTICE.txt`.
- Root `LICENSE` is the official Mozilla MPL-2.0 text for the eligible original files listed in `LICENSING.md`. It does not relicense any third-party material in this directory.

Verified origins:

- Mozilla: https://www.mozilla.org/media/MPL/2.0/index.txt
- ZXing 3.5.3 NOTICE: https://raw.githubusercontent.com/zxing/zxing/zxing-3.5.3/NOTICE
- egui/font crate: https://github.com/emilk/egui/tree/0.33.3/crates/epaint_default_fonts
- ring 0.17.14: https://github.com/briansmith/ring/tree/0.17.14

`python3 tools/check_licenses.py` checks the recorded copies; Android builds also check the packaged copies against their canonical source files. No internet connection is required for these checks. The hashes establish local copy consistency, not a substitute for provenance/rights review.

This directory preserves upstream font, crypto, Cargo and Rust standard-library/toolchain notices. It is one part of the release materials, not a standalone corresponding-source offer or a guarantee of legal clearance. Use the per-release manifest and matching Relay/vendor/native source archives for the actual distribution snapshot. See `THIRD_PARTY_NOTICES.md` and `packaging/DISTRIBUTION.md`.

`BRANDING_ASSET_MANIFEST.json` is Relay asset hash metadata, not an upstream license; the applicable adopted branding permissions are in root `BRANDING.md`. `RELAY-SOURCE.txt` is the local-build source-availability notice; it does not claim public binary obligations are already satisfied.

Rust runtime/library notices from Fedora rust/rust-std-static 1.99.0-1.fc43 are preserved under rust-toolchain-1.99.0/. Their installed paths and hashes are recorded in PROVENANCE.json; these upstream texts are not Relay MPL-covered source. Artwork copyright permission is separately in ARTWORK_LICENSE.md (MIT/Expat v1.0), and product identity in BRANDING.md v1.1, adopted effective 2026-11-19.
