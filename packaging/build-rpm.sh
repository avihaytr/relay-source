#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
set -euo pipefail
cd "$(dirname "$0")/.."
python3 tools/check_licenses.py
rpm_work=${1:?Usage: packaging/build-rpm.sh ABSOLUTE_RPM_DIRECTORY VENDOR_ARCHIVE}
vendor_archive=${2:?Provide the exact matching Cargo vendor archive}
mkdir -p "$rpm_work"/{SOURCES,SPECS,BUILD,BUILDROOT,RPMS,SRPMS}
python3 packaging/make-source.py --out "$rpm_work/SOURCES/relay-0.1.0-alpha.1-source.tar.xz"
cp "$vendor_archive" "$rpm_work/SOURCES/relay-0.1.0-alpha.1-cargo-vendor.tar.xz"
cp packaging/relay.spec "$rpm_work/SPECS/relay.spec"
rpmbuild -ba --define "_topdir $rpm_work" --define "_buildhost reproducible.invalid" \
  --define "packager Relay Maintainer" --define "vendor Relay Project" "$rpm_work/SPECS/relay.spec"
