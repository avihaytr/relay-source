# SPDX-License-Identifier: MPL-2.0
Name: relay
Version: 0.1.0~alpha.1
Release: 1%{?dist}
Summary: Local screen mirroring between Android and Linux
# Original source remains MPL; the combined GPL media build and separate assets retain their terms.
# Linux code/fonts: Apache and MIT are both required; no BSL-only crate is in this target.
# Artwork copyright is MIT; product-identity terms are separate trademark/origin permissions.
# MPL source and GPL-covered larger-work obligations both remain applicable.
License: MPL-2.0 AND GPL-3.0-or-later AND Apache-2.0 AND MIT AND ISC AND BSD-3-Clause AND BSD-2-Clause AND OFL-1.1 AND Ubuntu-font-1.0 AND WTFPL AND Zlib AND Unicode-3.0 AND CC0-1.0
URL: https://github.com/avihaytr/relay-source
Source0: relay-0.1.0-alpha.1-source.tar.xz
Source1: relay-0.1.0-alpha.1-cargo-vendor.tar.xz
ExclusiveArch: x86_64
BuildRequires: cargo
BuildRequires: python3
BuildRequires: rust
BuildRequires: gcc
BuildRequires: clang
BuildRequires: pkgconf-pkg-config
BuildRequires: alsa-lib-devel
BuildRequires: ffmpeg-devel
BuildRequires: desktop-file-utils
Requires: fedora-release-common = 43
Requires: openssl
Requires: avahi-tools
Requires: avahi
Requires: python3-gobject
Requires: gstreamer1-plugins-base
Requires: gstreamer1-plugins-bad-free
Requires: gstreamer1-plugins-ugly
Requires: pipewire-gstreamer
Requires: xdg-desktop-portal
Requires: xdg-desktop-portal-gnome
Requires: vulkan-loader
Recommends: scrcpy
Recommends: android-tools
%global debug_package %{nil}
%global _build_id_links none
%global relay_target %{_topdir}/CARGO_TARGET

%description
Relay mirrors Android screens to Linux and Linux windows/screens to Android.
This preview is qualified only for Fedora 43 x86_64. It uses the production
GPL-enabled FFmpeg/x264 pipeline and system native libraries, not the experimental
LGPL media prototype. Required corresponding sources and notices accompany distribution.

%prep
%setup -q -n relay-0.1.0-alpha.1
%{__tar} -xf %{SOURCE1}
mkdir -p .cargo
cat > .cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"
[source.vendored-sources]
directory = "vendor"
EOF

%build
python3 tools/check_licenses.py
export RUSTFLAGS="--remap-path-prefix=$PWD=/usr/src/relay --remap-path-prefix=$HOME=/build/home"
export SOURCE_DATE_EPOCH=1795039390
export CARGO_BUILD_JOBS=4
export CARGO_TARGET_DIR=%{relay_target}
cargo build --release --locked --offline

%check
export CARGO_BUILD_JOBS=4
export CARGO_TARGET_DIR=%{relay_target}
cargo test --release --locked --offline
desktop-file-validate packaging/relay.desktop

%install
install -Dpm0755 %{relay_target}/release/relay %{buildroot}%{_bindir}/relay
install -Dpm0644 packaging/relay.desktop %{buildroot}%{_datadir}/applications/relay.desktop
install -Dpm0644 assets/relay-icon.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/relay.svg

%files
%license LICENSE LICENSING.md ARTWORK_LICENSE.md BRANDING.md THIRD_PARTY_NOTICES.md assets/FONT-LICENSE.txt licenses
%doc SECURITY.md packaging/INSTALL.md packaging/DISTRIBUTION.md packaging/ANDROID_SIGNING.md
%{_bindir}/relay
%{_datadir}/applications/relay.desktop
%{_datadir}/icons/hicolor/scalable/apps/relay.svg

%changelog
* Thu Nov 19 2026 Relay Maintainer <95977499+avihaytr@users.noreply.github.com> - 0.1.0~alpha.1-1
- First local preview package candidate; not an approved public binary release.
