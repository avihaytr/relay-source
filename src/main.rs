// SPDX-License-Identifier: MPL-2.0
mod desktop;
mod companion;
mod view;
mod mirror;
mod audio;
mod pc_share;
mod profile;
mod pairing;
mod trust;
use egui::{self, *};
use std::{path::PathBuf, process::{Child, Command, Stdio}, sync::mpsc, time::{Duration, Instant}};
use serde::{Serialize, Deserialize};

const BG: Color32 = Color32::from_rgb(246,248,252);
const PANEL: Color32 = Color32::WHITE;
const TEXT: Color32 = Color32::from_rgb(30,43,68);
const GREEN: Color32 = Color32::from_rgb(52,113,246);
#[derive(Clone,Debug)] struct Device { serial:String, name:String, state:String, wifi:bool }
#[derive(Clone,Serialize,Deserialize)] struct Settings { #[serde(default="default_device_name")] device_name:String,#[serde(default="default_wireless_profile")] wireless_profile:u32,size:u32, fps:u32, bitrate:u32, audio:bool, control:bool, fullscreen:bool, hardware:bool, folder:String }
impl Default for Settings { fn default()->Self { Self {device_name:default_device_name(),wireless_profile:2,size:1920,fps:60,bitrate:8,audio:true,control:true,fullscreen:false,hardware:true,folder:format!("{}/captures",env!("CARGO_MANIFEST_DIR"))} } }
fn default_wireless_profile()->u32{2}
fn default_device_name()->String{profile::random_name()}
enum Event { Scan(Result<Vec<Device>,String>), Message(String) }
fn run(program:&str,args:&[&str])->Result<std::process::Output,String> {run_with_input(program,args,None)}
fn run_with_input(program:&str,args:&[&str],input:Option<&str>)->Result<std::process::Output,String> {
    use std::io::{Read,Write};
    let mut child=Command::new(program).args(args).stdin(if input.is_some(){Stdio::piped()}else{Stdio::null()}).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e|format!("Could not run {program}: {e}"))?;
    if let Some(input)=input{if let Some(mut stdin)=child.stdin.take(){let _=writeln!(stdin,"{input}");}}
    let mut stdout=child.stdout.take().unwrap(); let mut stderr=child.stderr.take().unwrap();
    let out=std::thread::spawn(move||{let mut b=Vec::new();let _=stdout.read_to_end(&mut b);b});
    let err=std::thread::spawn(move||{let mut b=Vec::new();let _=stderr.read_to_end(&mut b);b});
    let start=Instant::now();
    loop { match child.try_wait() { Ok(Some(status))=>return Ok(std::process::Output{status,stdout:out.join().unwrap_or_default(),stderr:err.join().unwrap_or_default()}),Ok(None)=>{},Err(e)=>return Err(e.to_string()) }
        if start.elapsed()>Duration::from_secs(8) { let _=child.kill();let _=child.wait();return Err(format!("{program} timed out. Check your connection and try again.")); }
        std::thread::sleep(Duration::from_millis(80));
    }
}
fn scan()->Result<Vec<Device>,String> { let o=run("adb", &["devices","-l"])?; if !o.status.success(){return Err("Device discovery unavailable. Check ADB permissions and USB access.".into())} Ok(parse_devices(&String::from_utf8_lossy(&o.stdout))) }
fn parse_devices(s:&str)->Vec<Device> { s.lines().skip(1).filter_map(|l|{let p:Vec<_>=l.split_whitespace().collect();if p.len()<2{return None} Some(Device{serial:p[0].into(),state:p[1].into(),name:p.iter().find_map(|x|x.strip_prefix("model:")).unwrap_or(p[0]).replace('_'," "),wifi:p[0].contains(':')||p[0].contains("._tcp")})}).collect() }
fn stamp()->u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64 }
struct Relay {mirror_mode:bool,mirror_geometry:Option<([usize;2],u8)>,mirror_rotation:u8,menu_size:Vec2,menu_maximized:bool,fullscreen:bool,last_image:Option<std::sync::Arc<ColorImage>>, companion:Option<companion::Companion>, qr_install:bool,easy:bool, texture:Option<egui::TextureHandle>, settings:Settings, devices:Vec<Device>, selected:Option<String>, tx:mpsc::Sender<Event>, rx:mpsc::Receiver<Event>, scanning:bool,last_scan:Instant,page:usize, tab:usize, status:String, details:String, child:Option<Child>, recording:bool, started:Instant, dialog:bool, address:String,pair_address:String,code:String,help:bool, frame:u32, screenshot:Option<PathBuf>,shot_requested:bool, saved:bool, log:Option<PathBuf>, capture_files:Vec<PathBuf> }
impl Relay {
fn new(cc:&desktop::CreationContext<'_>,screenshot:Option<PathBuf>)->Self {
 let mut style=(*cc.egui_ctx.style()).clone();style.visuals=Visuals::light();style.visuals.panel_fill=BG;style.visuals.window_fill=PANEL;style.visuals.override_text_color=Some(TEXT);style.spacing.button_padding=vec2(14.,10.);style.visuals.widgets.inactive.bg_fill=Color32::from_rgb(244,246,250);style.visuals.widgets.inactive.corner_radius=CornerRadius::same(7);style.visuals.selection.bg_fill=Color32::from_rgb(224,235,255);cc.egui_ctx.set_style(style);
 let mut fonts=FontDefinitions::default(); fonts.font_data.insert("interface".into(),FontData::from_static(include_bytes!("../assets/interface.ttf")).into());fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0,"interface".into());cc.egui_ctx.set_fonts(fonts);
 let (tx,rx)=mpsc::channel();let settings=cc.storage.and_then(|s|s.get_string("settings")).and_then(|s|serde_json::from_str(&s).ok()).unwrap_or_default();
 let mut a=Self{mirror_mode:false,mirror_geometry:None,mirror_rotation:0,menu_size:vec2(1240.,850.),menu_maximized:false,fullscreen:false,last_image:None,qr_install:false,texture:None,companion:None,easy:std::env::args().any(|a|a=="--connect-phone"),settings,devices:vec![],selected:None,tx,rx,scanning:false,last_scan:Instant::now(),page:0,tab:2,status:"Ready when you are".into(),details:String::new(),child:None,recording:false,started:Instant::now(),dialog:false,address:String::new(),pair_address:String::new(),code:String::new(),help:false,frame:0,screenshot,shot_requested:false,saved:false,log:None,capture_files:vec![]};a.refresh();a
}
fn apply_wireless_settings(&mut self){
 self.settings.device_name=profile::device_name(&self.settings.device_name);self.settings.wireless_profile=self.settings.wireless_profile.min(2);
 let path=PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"),"/settings.json"));
 let result=(||->Result<(),String>{let settings=serde_json::to_string(&self.settings).map_err(|e|e.to_string())?;let bytes=serde_json::to_vec(&serde_json::json!({"settings":settings})).map_err(|e|e.to_string())?;let temporary=path.with_extension("json.new");std::fs::write(&temporary,bytes).map_err(|e|e.to_string())?;std::fs::rename(temporary,path).map_err(|e|e.to_string())})();
 if let Err(error)=result{self.status=format!("Could not save settings: {error}");return;}
 self.companion=None;self.easy=true;self.status="Device name and Wi-Fi quality saved".into();
}
fn refresh(&mut self){if self.scanning{return}self.scanning=true;self.last_scan=Instant::now();let tx=self.tx.clone();std::thread::spawn(move||{let _=tx.send(Event::Scan(scan()));});}
fn files(&mut self) { self.capture_files=std::fs::read_dir(&self.settings.folder).into_iter().flatten().flatten().map(|e|e.path()).filter(|p|p.extension().is_some_and(|e|e=="png"||e=="mkv")).collect();self.capture_files.sort();self.capture_files.reverse(); }
fn chosen(&self)->Option<&Device>{ self.devices.iter().find(|d|Some(&d.serial)==self.selected.as_ref()&&d.state=="device") }
fn start(&mut self,record:bool) {if self.child.is_some(){return}let Some(d)=self.chosen().cloned() else {self.status="Connect and authorize a device first".into();return};
 let mut args=vec!["--serial".into(),d.serial,"--window-title".into(),"Relay · Android mirror".into(),format!("--max-size={}",self.settings.size),format!("--max-fps={}",self.settings.fps),format!("--video-bit-rate={}M",self.settings.bitrate)];
 if !self.settings.audio{args.push("--no-audio".into());}if !self.settings.control{args.push("--no-control".into());}if self.settings.fullscreen{args.push("--fullscreen".into());}
 let help=Command::new("scrcpy").arg("--help").output().ok().map(|x|String::from_utf8_lossy(&x.stdout).to_string()).unwrap_or_default();if help.contains("--hwdec"){args.push(format!("--hwdec={}",if self.settings.hardware{"auto"}else{"disabled"}));}
 if let Err(e)=std::fs::create_dir_all(&self.settings.folder){self.status=format!("Cannot create capture folder: {e}");return}
 if record {args.push(format!("--record={}/Recording-{}.mkv",self.settings.folder,stamp()));}
 let log=PathBuf::from(&self.settings.folder).join("session.log");let Ok(file)=std::fs::File::create(&log) else {self.status="Cannot write the capture folder".into();return};let Ok(err)=file.try_clone() else{return};
 match Command::new("scrcpy").args(&args).stdout(Stdio::from(file)).stderr(Stdio::from(err)).spawn(){Ok(c)=>{self.child=Some(c);self.log=Some(log);self.recording=record;self.started=Instant::now();self.status="Opening the native mirror window…".into()},Err(e)=>self.status=format!("Install scrcpy to start mirroring. {e}")}
}
fn stop(&mut self){if let Some(c)=&self.child {let _=Command::new("kill").args(["-INT",&c.id().to_string()]).status();self.status="Finishing your session…".into();}}
fn end_native_session(&mut self,ctx:&Context){self.companion=None;self.texture=None;self.last_image=None;self.easy=true;self.status="Mirroring ended".into();mirror::sync_window(self,ctx);}
fn capture_native(&mut self){
 let Some(frame)=self.last_image.clone() else{return};let folder=self.settings.folder.clone();let tx=self.tx.clone();
 std::thread::spawn(move||{let result=(||->Result<String,String>{std::fs::create_dir_all(&folder).map_err(|e|e.to_string())?;let path=PathBuf::from(&folder).join(format!("Screenshot-{}.png",stamp()));let bytes:Vec<u8>=frame.pixels.iter().flat_map(|c|c.to_array()).collect();image::save_buffer(path,&bytes,frame.width()as u32,frame.height()as u32,image::ColorType::Rgba8).map_err(|e|e.to_string())?;Ok("Screenshot saved to My captures".into())})();let _=tx.send(Event::Message(result.unwrap_or_else(|e|e)));});
}
fn capture(&mut self){let Some(d)=self.chosen().cloned() else{self.status="Connect a device to take a screenshot".into();return};let tx=self.tx.clone();let folder=self.settings.folder.clone();std::thread::spawn(move||{let result=(||{std::fs::create_dir_all(&folder).map_err(|e|e.to_string())?;let o=run("adb", &["-s",&d.serial,"exec-out","screencap","-p"])?;if !o.status.success()||!o.stdout.starts_with(b"\x89PNG"){return Err("Screenshot failed. Unlock and authorize the device.".into())}let path=format!("{folder}/Screenshot-{}.png",stamp());std::fs::write(&path,o.stdout).map_err(|e|e.to_string())?;Ok("Screenshot saved to your capture folder".into())})();let _=tx.send(Event::Message(result.unwrap_or_else(|e|e)));});}
fn connect(&mut self,pair:bool){let addr=if pair{self.pair_address.trim()}else{self.address.trim()}.to_string();if addr.parse::<std::net::SocketAddr>().is_err(){self.status="Enter a valid IP address and port, for example 192.168.1.42:37123".into();return}if pair&&(self.code.len()!=6||!self.code.chars().all(|c|c.is_ascii_digit())){self.status="Enter the six-digit pairing code shown on your phone".into();return}let code=self.code.clone();self.code.clear();let tx=self.tx.clone();self.status="Connecting to your phone…".into();std::thread::spawn(move||{let args=if pair{vec!["pair",addr.as_str()]}else{vec!["connect",addr.as_str()]};let msg=match run_with_input("adb",&args,if pair{Some(&code)}else{None}){Ok(o)=>format!("{}{}",String::from_utf8_lossy(&o.stdout),String::from_utf8_lossy(&o.stderr)),Err(e)=>e};let msg=if pair&&!code.is_empty(){msg.replace(&code,"[redacted]")}else{msg};let _=tx.send(Event::Message(msg));let _=tx.send(Event::Scan(scan()));});}
fn poll(&mut self){while let Ok(e)=self.rx.try_recv(){match e {Event::Scan(r)=>{self.scanning=false;match r {Ok(d)=>{self.details=String::new();self.devices=d;if !self.devices.iter().any(|d|Some(&d.serial)==self.selected.as_ref()){self.selected=self.devices.iter().find(|d|d.state=="device").map(|d|d.serial.clone());}},Err(e)=>self.details=e}},Event::Message(s)=>{self.status=s;self.files();}}}if self.last_scan.elapsed()>Duration::from_secs(6){self.refresh();}if let Some(c)=&mut self.child {if let Ok(Some(s))=c.try_wait(){self.child=None;self.recording=false;if s.success(){self.status="Session ended. Your captures are ready.".into()}else{self.status="Mirror stopped. Check connection and session details.".into();self.details=self.log.as_ref().and_then(|p|std::fs::read_to_string(p).ok()).unwrap_or_default();}self.files();}}}
}
impl desktop::App for Relay {
fn save(&mut self,storage:&mut dyn desktop::Storage){if let Ok(s)=serde_json::to_string(&self.settings){storage.set_string("settings",s)}}
fn update(&mut self,ctx:&Context,_frame:&mut desktop::Frame){self.poll();
 if let Some(c)=&self.companion{if !c.live.load(std::sync::atomic::Ordering::Relaxed){self.texture=None;self.last_image=None;c.frame.lock().unwrap().take();}else if let Some(frame)=c.frame.lock().unwrap().take(){self.last_image=Some(frame.clone());if let Some(t)=&mut self.texture{t.set(frame,TextureOptions::LINEAR);}else{self.texture=Some(ctx.load_texture("phone-video",frame,TextureOptions::LINEAR));}}}
 if ctx.input(|i|i.key_pressed(Key::F11)){self.fullscreen=!self.fullscreen;ctx.send_viewport_cmd(ViewportCommand::Fullscreen(self.fullscreen));}
 if self.fullscreen&&ctx.input(|i|i.key_pressed(Key::Escape)){self.fullscreen=false;ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));}
 ctx.request_repaint_after(Duration::from_millis(if self.screenshot.is_some(){16}else{200}));
 mirror::sync_window(self,ctx);
 if self.mirror_mode{mirror::render(self,ctx);}else{view::render(self,ctx);}
 if let Some(approval)=self.companion.as_ref().and_then(|c|c.pairing.lock().unwrap().pending.clone()){
     let mut open=true;
     Window::new(if approval.pairing{"Approve new device pairing?"}else{"Approve this mirroring session?"}).open(&mut open).collapsible(false).resizable(false).show(ctx,|ui|{
         ui.label(format!("Phone name (not independently verified): {}",approval.name));
         if approval.pairing{ui.label("This request pairs the phone with Relay. No screen will be shared yet.");}
         else{ui.label(if approval.watching{"This phone requests to view a screen from this PC."}else{"This phone requests to share its screen to this PC."});}
         if let Some(code)=&approval.verification_code{ui.label(format!("Compare this verification code with the phone: {code}"));ui.label("Approve only if both codes match.");}
         ui.label(if approval.remember{"Approving will remember this device for future signed connections. You can revoke it in Trusted Devices."}else{"Approve only a session you initiated. New devices must scan this PC’s QR."});
         ui.label(format!("Expires in {} seconds",approval.expires.saturating_duration_since(Instant::now()).as_secs()));
         ui.horizontal(|ui|{if ui.button("Reject").clicked(){approval.decide(false);}if ui.button(if approval.pairing{"Approve pairing"}else{"Approve session"}).clicked(){approval.decide(true);}});
     });if !open{approval.decide(false);}
 }
 if self.dialog {let mut open=true;Window::new("Connect over Wi-Fi").open(&mut open).collapsible(false).resizable(false).default_width(460.).show(ctx,|ui|{ui.label("Connect your phone and computer to the same trusted network.");ui.add_space(14.);ui.label("On Android 11+: Developer options → Wireless debugging.");ui.label("Choose ‘Pair device with pairing code’. Use its IP and pairing port.");ui.add_space(10.);ui.label("Pairing address (IP:port)");ui.text_edit_singleline(&mut self.pair_address);ui.label("Six-digit pairing code");ui.add(TextEdit::singleline(&mut self.code).password(true));if ui.button("Pair device").clicked(){self.connect(true)}ui.add_space(12.);ui.separator();ui.label("Return to Wireless debugging for the connection IP and port.");ui.label("Connection address (IP:port)");ui.text_edit_singleline(&mut self.address);if ui.button("Connect device").clicked(){self.connect(false)}ui.add_space(12.);ui.label(RichText::new(&self.status).color(GREEN));ui.add_space(8.);ui.label("For older Android devices, connect via USB first, then enable\nTCP/IP debugging on your trusted network before connecting.");if ui.button("Enable legacy Wi-Fi on selected USB device").clicked(){if let Some(d)=self.chosen().cloned(){let tx=self.tx.clone();std::thread::spawn(move||{let msg=match run("adb",&["-s",&d.serial,"tcpip","5555"]){Ok(o)=>format!("{}{}",String::from_utf8_lossy(&o.stdout),String::from_utf8_lossy(&o.stderr)),Err(e)=>e};let _=tx.send(Event::Message(msg));});}else{self.status="First connect and authorize a USB device.".into()}}});self.dialog=open;}
 if self.help {let mut open=true;Window::new("Connection guide & session details").open(&mut open).default_width(540.).show(ctx,|ui|{ScrollArea::vertical().max_height(530.).show(ui,|ui|{ui.heading("Easy connection");ui.label("Install the companion from your own verified build. Open Relay on both devices.\nPair by scanning the PC QR, confirm on your phone, then approve pairing here.\nTap the paired PC on your phone and choose a mirroring direction when ready.\nPairing does not capture a screen; Android/desktop capture still needs OS consent.\nQR codes expire after two minutes.\n\nUSB and advanced Wi-Fi use scrcpy for audio, recording and remote control.");ui.add_space(14.);ui.heading("Advanced USB connection");ui.label("1. On your phone, open Settings → About phone. Tap Build number\n    seven times to unlock Developer options.\n2. Open Developer options and enable USB debugging.\n3. Connect a data-capable USB cable. Unlock the phone and tap Allow.\n4. Select your phone in Relay, then click Start mirroring.");ui.add_space(14.);ui.heading("In the mirror window");ui.label("F11  •  Toggle fullscreen\nAlt + Left / Right  •  Rotate the view\nAlt + R  •  Request device rotation\nRight click  •  Back     Middle click  •  Home\nMouse and keyboard input work when device control is enabled.");ui.add_space(14.);ui.label("Record starts a new recorded session. Stop the session to finish\nthe file safely. Audio and quality settings apply at session start.\nAudio support depends on your Android version and app restrictions.");if !self.details.is_empty(){ui.add_space(14.);ui.heading("Diagnostics");ui.add(Label::new(RichText::new(&self.details).monospace().size(11.)).wrap());}});});self.help=open;}
 self.frame+=1;if self.screenshot.is_some()&&self.frame>=120&&!self.shot_requested{ctx.send_viewport_cmd(ViewportCommand::Screenshot(UserData::default()));self.shot_requested=true;}
 let screenshot_events=ctx.input(|i|i.events.clone());for e in &screenshot_events{if let egui::Event::Screenshot{image,..}=e{if let Some(path)=&self.screenshot{if let Some(parent)=path.parent(){let _=std::fs::create_dir_all(parent);}let bytes:Vec<u8>=image.pixels.iter().flat_map(|c|c.to_array()).collect();match image::save_buffer(path,&bytes,image.width() as u32,image.height() as u32,image::ColorType::Rgba8){Ok(())=>self.saved=true,Err(e)=>{eprintln!("Screenshot failed: {e}");std::process::exit(1)}};ctx.send_viewport_cmd(ViewportCommand::Close);}}}
 }
 fn close_requested(&mut self,ctx:&Context)->bool{if self.mirror_mode{self.end_native_session(ctx);false}else if self.companion.as_ref().is_some_and(|c|c.desktop.pending.load(std::sync::atomic::Ordering::Relaxed)||c.desktop.active.load(std::sync::atomic::Ordering::Relaxed)){self.companion.as_ref().unwrap().desktop.stop();false}else{true}}
 fn on_exit(&mut self,_gl:Option<&()>){self.companion=None;self.stop();if let Some(c)=&mut self.child{for _ in 0..30{if c.try_wait().ok().flatten().is_some(){return}std::thread::sleep(Duration::from_millis(100));}let _=c.kill();let _=c.wait();}}
}
fn main()->desktop::Result {let mut args=std::env::args().skip(1);let mut screenshot=None;let mut headless=false;while let Some(a)=args.next(){if a=="--screenshot"||a=="--headless-screenshot"{headless=a=="--headless-screenshot";screenshot=args.next().map(PathBuf::from);if screenshot.is_none(){eprintln!("--screenshot requires a PNG path");std::process::exit(2)}}}
 if headless {return desktop::render_headless(screenshot.unwrap(),Box::new(|cc|Ok(Box::new(Relay::new(cc,None)))))}
 let mut viewport=ViewportBuilder::default().with_inner_size([1240.,850.]).with_min_inner_size([1080.,800.]).with_title("Relay — Android, on your desktop");if screenshot.is_some(){viewport=viewport.with_active(false);}
 #[cfg(target_os="windows")] if screenshot.is_some(){viewport=viewport.with_window_level(WindowLevel::AlwaysOnBottom);}
 let options=desktop::NativeOptions{viewport,background_render:screenshot.is_some(),event_loop_builder:if screenshot.is_some(){Some(Box::new(|builder|{
 #[cfg(target_os="macos")] {use winit::platform::macos::EventLoopBuilderExtMacOS;builder.with_activate_ignoring_other_apps(false);}
 #[cfg(not(target_os="macos"))] let _=builder;
 }))}else{None}};desktop::run_native("Relay",options,Box::new(move|cc|Ok(Box::new(Relay::new(cc,screenshot)))))
}
#[cfg(test)] mod tests {use super::*;#[test]fn device_states_and_connections(){let d=parse_devices("List of devices attached\nabc device product:p model:Pixel_9 device:p\n192.168.1.4:5555 device model:Tablet\nx unauthorized\ny offline\n");assert_eq!(d.len(),4);assert_eq!(d[0].name,"Pixel 9");assert!(!d[0].wifi);assert!(d[1].wifi);assert_eq!(d[2].state,"unauthorized");assert_eq!(d[3].state,"offline");}}
