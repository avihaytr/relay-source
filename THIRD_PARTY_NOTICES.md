# Third-party components and attribution

Relay's **explicitly marked original source files** are licensed under standard MPL-2.0. See [LICENSE](LICENSE) and the exact scope/exclusions in [LICENSING.md](LICENSING.md). This grant does not replace the terms below, license underlying artwork references, or clear the complete app for publication. No Exhibit B incompatibility restriction is applied to the marked source.

## Material carried in the repository or embedded by builds

| Component | Verified version, provenance and attribution | Terms / preserved notices |
|---|---|---|
| Noto Sans (`assets/interface.ttf`) | **2.015**, Copyright 2022 The Noto Project Authors; font metadata points to [notofonts/latin-greek-cyrillic](https://github.com/notofonts/latin-greek-cyrillic). Embedded by `src/main.rs`. Exact upstream byte import still needs a reproducibility record. | **OFL-1.1**, unchanged [assets/FONT-LICENSE.txt](assets/FONT-LICENSE.txt). Preserve copyright/license and applicable reserved font names. This is not MPL-covered artwork. |
| ZXing Core (`android/lib/zxing-core.jar`) | **com.google.zxing:core:3.5.3**, ZXing Authors, with Barcode4J-derived PDF417 code. JAR SHA-256 `8d8064c1636fdaef7189dd9055c7d59950a8940a12f2293956446ec3c109fd82`; matches the Maven artifact inspected in the audit. [Upstream release](https://github.com/zxing/zxing/tree/zxing-3.5.3). | **Apache-2.0**: [ZXING-LICENSE.txt](android/lib/ZXING-LICENSE.txt) and the full unchanged upstream [ZXING-NOTICE.txt](android/lib/ZXING-NOTICE.txt). The NOTICE credits Jeremias Märki (2002–2010), Dietmar Bürkle (2005–2006) and Barcode4J contributors; its JCommander/Cedric Beust section is conservatively retained, not a claim that JCommander ships in Core. |
| egui embedded default fonts | **epaint_default_fonts 0.33.3**, locked in `Cargo.lock`; enabled by `FontDefinitions::default()` even with added Noto Sans. [Upstream crate](https://github.com/emilk/egui/tree/0.33.3/crates/epaint_default_fonts). | Package declaration `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0`; use the per-font details below, not the code crate's MIT/Apache license alone. |
| Hack-Regular | **3.003**, Source Foundry Authors (2018), Bitstream Inc. (2003), with DejaVu public-domain contributions; metadata from the packaged TTF. | MIT **and** relevant Bitstream Vera notice/naming conditions; preserve the complete [Hack-Regular.txt](licenses/epaint_default_fonts-0.33.3/fonts/Hack-Regular.txt). |
| NotoEmoji-Regular | **1.05 uh**, Copyright 2013 Google Inc., from the packaged TTF. | OFL-1.1; [OFL.txt](licenses/epaint_default_fonts-0.33.3/fonts/OFL.txt) and [font copyrights](licenses/epaint_default_fonts-0.33.3/FONT-COPYRIGHTS.txt). |
| Ubuntu-Light | **0.83**, Copyright 2011 Canonical Ltd., from the packaged TTF. | Ubuntu Font Licence 1.0; preserve [UFL.txt](licenses/epaint_default_fonts-0.33.3/fonts/UFL.txt), attribution and applicable font naming/modification restrictions. Do not relicense the font MPL or GPL. |
| emoji-icon-font | **1.1**, Copyright (c) 2014 John Slegers, as credited by the packaged MIT notice. | [Upstream MIT notice](licenses/epaint_default_fonts-0.33.3/fonts/emoji-icon-font-mit-license.txt). |

Font software licenses remain attached to the fonts; they do not license Relay code under those font licenses. Embedding a font does not remove its own distribution conditions. Copied license texts remain unchanged; [licenses/PROVENANCE.json](licenses/PROVENANCE.json) records their versions/origin paths and hashes. These copies do not constitute a complete binary-license bundle.

### Android packaging

`android/build.sh` copies the canonical ZXing license and full NOTICE into `assets/third_party/`, together with MPL-2.0, Relay's scope document and its local-build source-availability notice. The build checks the final APK entries byte for byte. ZXing is dexed unmodified; no JAR source file is relicensed. Apache's required notices remain applicable to that third-party code.

The local instrumentation APK packages the MPL/scope/source notice for its original test code; it does not bundle ZXing classes. No licensing preparation installs or distributes either APK. A public APK still needs an approved, accessible source-delivery location for that exact build and review of applicable asset permissions and any unresolved third-party rights. The packaged development notice explicitly does not claim those release conditions are already satisfied.

## Rust dependencies

Exact versions and registry archive checksums for the 408 locked external name/version pairs are in [Cargo.lock](Cargo.lock). This includes inactive platform/build dependencies, not just the Linux binary. The following table highlights direct dependencies; source publication does not bundle their compiled copies. Binary releases require a complete enabled dependency and notice inventory.

| Rust component (locked version) | Declared license | Upstream attribution/source |
|---|---|---|
| egui, egui-wgpu, egui-winit **0.33.3** | MIT OR Apache-2.0 | [egui contributors](https://github.com/emilk/egui); fonts have the separate terms above. |
| winit **0.30.13** | Apache-2.0 | [rust-windowing/winit](https://github.com/rust-windowing/winit), upstream LICENSE/file notices. |
| wgpu **27.0.1** | MIT OR Apache-2.0 | [gfx-rs/wgpu](https://github.com/gfx-rs/wgpu). |
| pollster **0.4.0** | Apache-2.0/MIT (legacy alternative notation) | [pollster](https://github.com/zesterer/pollster), Joshua Barretto's packaged MIT notice and LICENSE-APACHE. |
| image **0.25.10** | MIT OR Apache-2.0 | [image-rs/image](https://github.com/image-rs/image). |
| serde **1.0.229**, serde_json **1.0.154** | MIT OR Apache-2.0 | [serde](https://github.com/serde-rs/serde), [serde_json](https://github.com/serde-rs/json). |
| rustls **0.23.46** | Apache-2.0 OR ISC OR MIT | [rustls](https://github.com/rustls/rustls); preserve the selected license's upstream notices, including Joseph Birr-Pixton attribution where applicable. |
| qrcode **0.14.1** | MIT OR Apache-2.0 | [qrcode-rust](https://github.com/kennytm/qrcode-rust). |
| ffmpeg-next **7.1.0**, ffmpeg-sys-next **7.1.3** | WTFPL | [rust-ffmpeg](https://github.com/zmwangx/rust-ffmpeg), [rust-ffmpeg-sys](https://github.com/zmwangx/rust-ffmpeg-sys). **Wrapper licenses do not license FFmpeg.** |
| cpal **0.15.3** | Apache-2.0 | [RustAudio/cpal](https://github.com/RustAudio/cpal). |
| ashpd **0.14.0** | MIT | [ashpd](https://github.com/bilelmoussaoui/ashpd), Bilal Elmoussaoui's packaged notice. |
| tokio **1.54.1** | MIT | [tokio](https://github.com/tokio-rs/tokio). |
| libc **0.2.191** | MIT OR Apache-2.0 | [rust-lang/libc](https://github.com/rust-lang/libc). |
| subtle **2.6.1** | BSD-3-Clause | [subtle](https://github.com/dalek-cryptography/subtle); packaged LICENSE credits Isis Agora Lovecruft, Henry de Valence and the other named upstream holders. |
| ring **0.17.14** | Apache-2.0 **AND** ISC | [ring](https://github.com/briansmith/ring); Brian Smith's ISC notice, BoringSSL-derived files and nested once_cell notices. Preserved guide/texts: [licenses/ring-0.17.14](licenses/ring-0.17.14). |

An **OR** expression permits a license choice; an **AND** expression requires the applicable combined notices/terms. Do not discard original licensing alternatives or infer copyright ownership from this summary. Individual copyright statements are in upstream file/license notices; the table is an inventory, not their replacement. A binary notices bundle must record the actual selected alternatives and incorporated files.

Transitive declarations include BSD-2/3-Clause, ISC, Zlib, BSL-1.0, Unicode-3.0, CC0/public-domain/Unlicense alternatives and LLVM exceptions. Some offer LGPL alternatives alongside MIT/Apache. Required Unicode/data, font and nested crypto notices must be preserved as applicable. Root COPYING/NOTICE texts alone may not capture every file-specific notice. Before any desktop binary release, assemble and audit the full enabled Rust/toolchain/native notice and source closure; that work is **not complete** here.

## Native dependencies and optional programs

These are observed development-system components, not bundled program copies in this repository. Versions/configuration are not a portable release specification. Their own licensing remains applicable to linked/combined distributions; dynamic linking or system installation is not a blanket exemption.

| Component / observed version | License evidence and distribution considerations |
|---|---|
| FFmpeg **7.1.5-1.fc43** (`ffmpeg-libs`, `libavcodec-freeworld`) | `avcodec_license()`, `avutil_license()` and `swscale_license()` report **GPL version 3 or later**. The build enables GPL/version3, x264 and x265, uses shared libraries, and showed no `--enable-nonfree`. Source RPM: `ffmpeg-7.1.5-1.fc43.src.rpm`. [FFmpeg licensing](https://ffmpeg.org/legal.html), [7.1.5 terms](https://github.com/FFmpeg/FFmpeg/blob/n7.1.5/LICENSE.md). GPLv3 corresponding-source, notice and combined-work obligations remain unresolved for release. |
| GStreamer core/base **1.26.11**, PyGObject **3.54.5** | LGPL-2.1-or-later framework/bindings; most selected conversion/rate/parse/appsink plugins report LGPL. Exact file/version matters; preserve source, replacement/relinking and notices as required by delivery. [GStreamer licensing guidance](https://gstreamer.freedesktop.org/documentation/application-development/appendix/licensing.html). |
| x264 **0.165-2.20250608gitb35605ac.fc43**, `x264enc` **1.26.11** | Actual plugin reports **GPL**. x264 snapshot is GPL-2.0-or-later with a separately offered commercial route; no commercial grant is established. The wrapper's Library-GPL-v2-or-later header/RPM summary does not make the loaded x264 combination LGPL-only. [x264 snapshot header](https://github.com/mirror/x264/blob/b35605ac/common/common.h). The Python helper loads it in-process; its boundary with Rust needs legal review. |
| PipeWire / `pipewiresrc` **1.4.11** | MIT / plugin reports MIT/X11; permission-scoped portal transport. |
| ALSA **1.2.16.1**, Vulkan loader **1.4.341.0** | ALSA LGPL-2.1-or-later; Vulkan loader Apache-2.0. Graphics ICD/driver and portal backend terms are environment-specific. |
| OpenSSL CLI **3.5.9**, Avahi tools **0.9~rc2** | OpenSSL Apache-2.0; Avahi package includes LGPL/BSD/MIT components. External tools, not relicensed or copied here. |
| Python **3.14.8**, Android API **35** / build tools **35.0.0**, Java toolchain | External tools/runtime/framework, not app-owned code. Their separate terms apply if those components are redistributed; compilation alone does not relicense Relay output. Android uses platform MediaCodec/MediaProjection/Keystore, not bundled FFmpeg/x264. |
| Optional scrcpy **5.0.1**, ADB **1.0.41 / android-tools 37.0.0p1** | [scrcpy source](https://github.com/Genymobile/scrcpy) Apache-2.0; its binary has its own FFmpeg/SDL closure. [ADB](https://android.googlesource.com/platform/packages/modules/adb/) and distro android-tools include multiple component licenses. Invoked as external programs; bundling requires a separate inventory, notices and source/compliance review. |

Package-wide license expressions can include documentation/tools not present in a particular ELF; no blanket conclusion about every linked file is made. H.264 patents, proprietary driver terms and the exact GPL reach across the helper boundary require separate review. A commercial x264 license would not relicense the existing GPL FFmpeg build.

`tests/fixtures/keyframe.h264` contains the standard x264 encoder identification and copyright string in its H.264 SEI payload. It is retained in the synthetic test stream; see [the fixture provenance record](tests/fixtures/README.md). This preserves the embedded attribution and is not a conclusion about the license of generated bitstream data.

## Branding review and approved test data

The app icon was generated with ChatGPT using one previous-Relay design reference, refined to an icon-only/background-removed output, then redrawn as SVG by Codex. The desktop and launcher PNGs reproduce from that SVG; the Android vector is a simplified rendition. [The active asset manifest](licenses/BRANDING_ASSET_MANIFEST.json) records hashes. Independent reference rights and exclusive copyright in generated output are not conclusively verified. Branding remains outside MPL; [BRANDING.md](BRANDING.md) contains the adopted separate policy, version 1.0 effective 2026-11-18. Adoption grants only rights held by the grantor and does not establish independently verified third-party clearance. The approved procedural drawing code in `src/view.rs` remains MPL-covered.

The public Java proof vector `tests/fixtures/java-p256-proof.txt` is explicitly covered by MPL-2.0 in [LICENSING.md](LICENSING.md); it remains three lines so existing test readers need no format change. The synthetic H.264 fixture is explicitly included under MPL-2.0 to the extent rights exist, with explicit scope recorded in LICENSING.md; [its reproducible provenance and embedded attribution](tests/fixtures/README.md) are retained. Output licensing and H.264 patent questions are not settled by this attribution record.

**Publication remains gated:** review the final source candidate and resolve any identified third-party rights concerns before authorizing source publication. Complete binary-specific source delivery, notices/GPL compliance and native-closure review before distributing builds. No binary distribution is approved by these notices.
