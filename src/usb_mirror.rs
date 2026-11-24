// SPDX-License-Identifier: MPL-2.0
//! Hosts the scrcpy video window inside Relay's mirror controls on X11.
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    time::{Duration, Instant},
};
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{Atom, AtomEnum, ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt, EventMask, InputFocus, MapState, Window},
    },
    rust_connection::RustConnection,
};

pub(crate) const HOST_WINDOW: &str = "relay-x11-host-window";

pub(crate) struct UsbMirror {
    connection: RustConnection,
    root: Window,
    host: Window,
    title: String,
    net_wm_name: Atom,
    wm_state: Atom,
    deadline: Instant,
    withdrawing: Option<Window>,
    window: Option<Window>,
    initial_size: [usize; 2],
    video_size: Option<[usize; 2]>,
    log: Option<BufReader<File>>,
    log_line: Vec<u8>,
    bounds: Option<[i32; 4]>,
}

impl UsbMirror {
    pub(crate) fn new(ctx: &egui::Context, title: String) -> Option<Self> {
        let host = ctx.data(|data| data.get_temp(egui::Id::new(HOST_WINDOW)))?;
        let (connection, screen) = x11rb::connect(None).ok()?;
        let root = connection.setup().roots.get(screen)?.root;
        let net_wm_name = connection.intern_atom(false, b"_NET_WM_NAME").ok()?.reply().ok()?.atom;
        let wm_state = connection.intern_atom(false, b"WM_STATE").ok()?.reply().ok()?.atom;
        Some(Self {
            connection, root, host, title, net_wm_name, wm_state,
            deadline: Instant::now() + Duration::from_secs(15),
            withdrawing: None,
            window: None,
            initial_size: [0, 0],
            video_size: None,
            log: None,
            log_line: Vec::new(),
            bounds: None,
        })
    }

    fn find_window(&self, parent: Window, depth: u8) -> Option<Window> {
        let children = self.connection.query_tree(parent).ok()?.reply().ok()?.children;
        for child in children.into_iter().rev() {
            if child == self.host {
                continue;
            }
            let matches_title = [self.net_wm_name, AtomEnum::WM_NAME.into()].into_iter().any(|atom| {
                self.connection.get_property(false, child, atom, AtomEnum::ANY, 0, 80)
                    .ok().and_then(|cookie| cookie.reply().ok())
                    .is_some_and(|reply| reply.value == self.title.as_bytes())
            });
            if matches_title {
                return Some(child);
            }
            if depth > 0 {
                if let Some(window) = self.find_window(child, depth - 1) {
                    return Some(window);
                }
            }
        }
        None
    }

    pub(crate) fn try_attach(&mut self) {
        self.update_video_size();
        if self.window.is_some() {
            return;
        }
        if self.expired() {
            self.restore_window();
            return;
        }
        if let Err(error) = self.attach_when_ready() {
            eprintln!("Relay: could not host scrcpy window: {error}");
            self.deadline = Instant::now();
            self.restore_window();
        }
    }

    pub(crate) fn follow_log(&mut self, path: &Path) {
        self.log = File::open(path).ok().map(BufReader::new);
    }

    fn update_video_size(&mut self) {
        let Some(log) = &mut self.log else { return };
        while let Ok(1..) = log.read_until(b'\n', &mut self.log_line) {
            if self.log_line.last() != Some(&b'\n') {
                break;
            }
            // A window manager can resize the SDL window before Relay hosts it.
            // Use scrcpy's decoded texture size, which also updates on rotation.
            if let Some(size) = parse_texture_size(&self.log_line) {
                self.video_size = Some(size);
            }
            self.log_line.clear();
        }
    }

    fn restore_window(&mut self) {
        if let Some(window) = self.withdrawing.take() {
            let _ = self.connection.map_window(window);
            let _ = self.connection.flush();
        }
    }

    fn attach_when_ready(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(window) = self.withdrawing {
            // Let the window manager release its frame before taking ownership.
            let state = self.connection.get_property(false, window, self.wm_state, AtomEnum::ANY, 0, 1)?.reply()?;
            let managed = state.value32().and_then(|mut values| values.next()).is_some_and(|state| state != 0);
            if !managed {
                self.attach(window)?;
                self.withdrawing = None;
            }
            return Ok(());
        }
        if let Some(window) = self.find_window(self.root, 1) {
            if self.connection.get_window_attributes(window)?.reply()?.map_state == MapState::VIEWABLE {
                self.connection.unmap_window(window)?.check()?;
                self.withdrawing = Some(window);
            }
        }
        Ok(())
    }

    fn attach(&mut self, window: Window) -> Result<(), Box<dyn std::error::Error>> {
        let geometry = self.connection.get_geometry(window)?.reply()?;
        self.connection.change_window_attributes(window,
            &ChangeWindowAttributesAux::new().event_mask(EventMask::STRUCTURE_NOTIFY))?.check()?;
        self.connection.reparent_window(window, self.host, 8, 36)?.check()?;
        if let Err(error) = self.connection.map_window(window)?.check() {
            let _ = self.connection.reparent_window(window, self.root, 0, 0);
            let _ = self.connection.map_window(window);
            let _ = self.connection.flush();
            return Err(error.into());
        }
        self.initial_size = [usize::from(geometry.width).max(1), usize::from(geometry.height).max(1)];
        self.window = Some(window);
        Ok(())
    }

    pub(crate) fn expired(&self) -> bool {
        self.window.is_none() && Instant::now() >= self.deadline
    }

    pub(crate) fn source(&self) -> Option<[usize; 2]> {
        self.window.map(|_| self.video_size.unwrap_or(self.initial_size))
    }

    pub(crate) fn position(&mut self, rect: egui::Rect, scale: f32) {
        let Some(window) = self.window else { return };
        self.sync_keyboard_focus(window);
        let bounds = [
            (rect.left() * scale).round() as i32,
            (rect.top() * scale).round() as i32,
            ((rect.width() * scale).round() as i32).max(1),
            ((rect.height() * scale).round() as i32).max(1),
        ];
        // scrcpy may resize its own window when Android rotates.
        while let Ok(Some(event)) = self.connection.poll_for_event() {
            if let Event::ConfigureNotify(event) = event {
                if event.window == window {
                    self.bounds = Some([
                        i32::from(event.x), i32::from(event.y),
                        i32::from(event.width), i32::from(event.height)
                    ]);
                }
            }
        }
        if self.bounds == Some(bounds) {
            return;
        }
        self.bounds = Some(bounds);
        let [x, y, width, height] = bounds;
        let changes = ConfigureWindowAux::new().x(x).y(y)
            .width(width as u32).height(height as u32);
        let _ = self.connection.configure_window(window, &changes);
        let _ = self.connection.flush();
    }

    fn sync_keyboard_focus(&self, window: Window) {
        let focus = self.connection.get_input_focus().ok()
            .and_then(|reply| reply.reply().ok()).map(|reply| reply.focus);
        if !matches!(focus, Some(focus) if focus == self.host || focus == window) {
            return;
        }
        let over_video = self.connection.query_pointer(self.host).ok()
            .and_then(|reply| reply.reply().ok())
            .is_some_and(|reply| reply.child == window);
        let target = if over_video { window } else { self.host };
        if focus != Some(target) {
            let _ = self.connection.set_input_focus(InputFocus::PARENT, target, x11rb::CURRENT_TIME);
            let _ = self.connection.flush();
        }
    }
}

fn parse_texture_size(line: &[u8]) -> Option<[usize; 2]> {
    let dimensions = std::str::from_utf8(line).ok()?.trim().strip_prefix("INFO: Texture: ")?;
    let (width, height) = dimensions.split_once('x')?;
    let size = [width.parse::<usize>().ok()?, height.parse::<usize>().ok()?];
    if size.contains(&0) {
        return None;
    }
    Some(size)
}
