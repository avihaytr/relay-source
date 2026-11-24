# Relay alpha.2 desktop preview

This portable Linux package targets Fedora 43 x86_64. Extract it into a folder you can keep and run:

    cd relay-0.1.0-alpha.2-fedora43-x86_64
    ./relay --connect-phone

Keep `relay` and `relay-scrcpy` together. For USB mirroring, install `android-tools`, SDL3 3.4 or later, and scrcpy 5.0.1 with its matching server at `/usr/share/scrcpy/scrcpy-server`. Use `SCRCPY_SERVER_PATH` to override that path. Enable USB debugging on the Android device and approve the device authorization prompt. Use the [Fedora dependencies from the first preview](https://github.com/avihaytr/relay-source/blob/v0.1.0-alpha.1/packaging/INSTALL.md) for the rest of the desktop app.

The alpha.1 Android APK remains the companion for wireless mirroring. Pinning a USB mirror on native Wayland offers a restart under XWayland when available. Pinning during a recording uses the desktop's window menu.

## Building and source materials

Build Relay with `cargo build --release --locked`. To build the bundled USB viewer, use the exact upstream scrcpy archive and follow the [patch instructions](README.md). The Fedora builds use system shared libraries. The [alpha.1 release](https://github.com/avihaytr/relay-source/releases/tag/v0.1.0-alpha.1) supplies the locked Cargo dependency sources and the Fedora FFmpeg and GStreamer native source archive. The alpha.2 Relay archive supplies the updated source, lockfile and scrcpy patch. The lockfile adds x11rb as a direct dependency from the existing dependency set.
