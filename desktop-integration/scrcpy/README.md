# Relay USB viewer

Apply toolbar.patch to scrcpy v5.0.1 (commit a60891aea193d92e7e5c3942700eca63f9d19a5f). The patch uses SDL 3.4 or newer for PNG capture. Build the client with the upstream dependencies and the matching scrcpy-server installed.

    git apply /path/to/toolbar.patch
    meson setup build --prefix=/usr --buildtype=release -Dcompile_server=false
    meson compile -C build

Install build/app/scrcpy beside the Relay executable as relay-scrcpy. Relay supplies the capture directory through RELAY_CAPTURE_DIR and falls back to the system scrcpy if the private client is absent. The private client uses scrcpy's normal renderer and input handling. Toolbar actions operate on its SDL window. On Linux, a native Wayland session can switch to standalone XWayland for direct pinning. The switch requires confirmation and is unavailable while recording. Subsequent pin clicks toggle the X11 always-on-top state.
