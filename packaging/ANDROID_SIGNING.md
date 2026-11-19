# Android release signing

The local preview uses an externally stored RSA-4096 PKCS#12 release keystore protected by a generated high-entropy password. The password is held in the desktop Secret Service, not a source file or command-line argument. Keystore directories are owner-only and the keystore mode is 0600. Neither the key nor its password is included in source/RPM/APK archives.

`packaging/build-android-release.py` compiles an unsigned, non-debuggable APK, zipaligns before signing, signs only with explicitly supplied external keys, and verifies the signature and packaged legal notices. There is no default release password or automatic fallback to the development key. Passwords reach apksigner through named environment variables; they are removed from the compiler child environment. Do not print those variables or use shell tracing while signing. An unlocked/compromised account can access its keyring and process environment; this is local key custody, not hardware-backed signing.

The approved local migration uses APK Signature Scheme v3 rotation, linking the previous development certificate to the new release certificate. The lineage contains certificates/signatures, not private keys. Both signing keys are necessary for the selected legacy-compatible signing layout; keep them private and preserve the lineage for future updates. Rotation is configured for API 28+. Android 8 uses the legacy signature; older-platform upgrade behavior must be qualified separately. Do not assume an arbitrary development keystore has this project's original signer.

Release certificate SHA-256:
`77ba0dcdab5b46f422c8e976f8ee4186a21c6f8e5d06f582149911e493894e81`

Before first public binary distribution, the owner must preserve an encrypted offline copy of the release keystore, recoverable keyring/password backup, the private legacy key needed for supported legacy signing, and the public lineage/certificate. A second file on the same machine is not an offline backup. Test restoration and future update signing on a separate disposable test setup. Loss of signing material can prevent updates; do not promise recovery without that test. Use a dedicated signing identity/role for future automation rather than publishing secrets in GitHub variables, logs or artifacts.

Example build inputs (set passwords securely outside shell history; do not paste secrets into chat):

```sh
python3 packaging/build-android-release.py \
  --keystore /secure/relay-release.p12 --password-env RELAY_RELEASE_PASSWORD \
  --old-keystore /secure/legacy-relay.keystore --old-password-env RELAY_OLD_PASSWORD \
  --lineage /secure/relay-signing.lineage --out /output/relay-0.1.0-alpha.1-android.apk
```

Paths are illustrative. The public source bundle is sufficient to compile unsigned code or sign independent builds with their own distinguishable identity; it does not include this project's private signing material. This document does not authorize public distribution or claim an external signing audit.

The owner-approved encrypted USB backup passed authenticated readback, member-hash verification and restored-key APK signing with the existing certificate. The test used owner-only RAM-backed temporary files, which were removed. A physical reconnect/clean-machine disaster-recovery exercise was not performed. Preserve an independent recoverable passphrase record; plaintext laptop notes are not secure independent backup storage. No signing identity was replaced by the licensing adoption.
