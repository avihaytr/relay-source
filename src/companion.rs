// SPDX-License-Identifier: MPL-2.0
//! Encrypted H.264 receiver with direct low-delay decoding and a single latest-frame slot.
use std::{io::Read, net::{TcpListener, TcpStream}, path::PathBuf, process::{Command, Child, Stdio}, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant}};
use rustls::{ServerConfig, ServerConnection, StreamOwned, pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer}};

pub struct Companion {
    pub peer_name:Arc<Mutex<String>>,
    pub desktop:Arc<crate::pc_share::DesktopShare>,
    pub audio:Arc<crate::audio::AudioControl>,
    advertisement:Option<Child>,
    pub discovery:bool,
    pub computer_name:String,
    pub pairing: Arc<Mutex<crate::pairing::Pairing>>,
    pub trusted: Arc<Mutex<crate::trust::Store>>,
    active_peer:Arc<Mutex<Option<String>>>,
    endpoint: String,
    pin: String,
    quality:u32,
    pub status: Arc<Mutex<String>>,
    pub frame: Arc<Mutex<Option<Arc<egui::ColorImage>>>>,
    pub live: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    active: Arc<Mutex<Option<TcpStream>>>,
}
fn command(args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("openssl").args(args).output().map_err(|e| e.to_string())?;
    if !out.status.success() { return Err(String::from_utf8_lossy(&out.stderr).into()); }
    Ok(out.stdout)
}
#[derive(Debug,PartialEq,Eq)]
enum PacketKind { Video, Audio, Info }
fn read_packet(input: &mut impl Read, bytes: &mut Vec<u8>) -> Result<PacketKind, String> {
    let mut header=[0;4];input.read_exact(&mut header).map_err(|e|e.to_string())?;
    let raw=u32::from_be_bytes(header);let audio=raw&0x8000_0000!=0;let info=raw&0x4000_0000!=0;let size=(raw&0x3fff_ffff) as usize;
    if size==0||size>4*1024*1024||(audio&&info)||(info&&size>4096)||(audio&&(size%4!=0||size>19_200)){return Err("Invalid video packet length".into());}
    bytes.resize(size,0);input.read_exact(bytes).map_err(|e|e.to_string())?;Ok(if audio{PacketKind::Audio}else if info{PacketKind::Info}else{PacketKind::Video})
}
fn video_decoder() -> Result<ffmpeg_next::decoder::Video,String> {
    use ffmpeg_next as ff;
    ff::init().map_err(|e|e.to_string())?;
    let codec=ff::decoder::find(ff::codec::Id::H264).ok_or("H.264 decoder missing")?;
    let mut context=ff::codec::context::Context::new_with_codec(codec);
    context.set_flags(ff::codec::Flags::LOW_DELAY);
    context.set_threading(ff::codec::threading::Config {kind:ff::codec::threading::Type::Slice,count:1,..Default::default()});
    context.decoder().video().map_err(|e|e.to_string())
}
fn update_peer(peer:&Mutex<String>,bytes:&[u8])->Result<(),String>{let info:serde_json::Value=serde_json::from_slice(bytes).map_err(|e|e.to_string())?;if let Some(name)=info.get("name").and_then(|v|v.as_str()){*peer.lock().unwrap()=crate::profile::device_name(name);}Ok(())}
impl Companion {
    pub fn forget_device(&self,id:&str)->Result<(),String>{
        let mut store=self.trusted.lock().unwrap();store.forget(id)?;
        if self.active_peer.lock().unwrap().as_deref()==Some(id){self.desktop.stop();if let Some(socket)=self.active.lock().unwrap().as_ref(){let _=socket.shutdown(std::net::Shutdown::Both);}}
        if let Some(pending)=&self.pairing.lock().unwrap().pending{if pending.device_id.as_deref()==Some(id){pending.decide(false);}}
        Ok(())
    }
    pub fn pairing_link(&self)->Result<String,String>{let mut pair=self.pairing.lock().unwrap();let code=pair.qr_code(Instant::now())?;let name=self.computer_name.bytes().map(|b|if b.is_ascii_alphanumeric()||b"-_.".contains(&b){(b as char).to_string()}else{format!("%{b:02X}")}).collect::<String>();Ok(format!("relay://{}/{}/{}?v=6&q={}&computer={}",self.endpoint,code,self.pin,self.quality,name))}
    pub fn pairing_pin(&self)->Result<String,String>{Ok(self.pairing.lock().unwrap().pin_code(Instant::now())?.to_string())}
    pub fn start(ctx: egui::Context,name:String,wireless_profile:u32) -> Result<Self, String> {
        let root = PathBuf::from(std::env::var("HOME").map_err(|e| e.to_string())?).join(".local/share/relay");
        crate::trust::private_directory(&root)?;
        let cert = root.join("receiver.der"); let key = root.join("receiver-key.der");
        if cert.exists()!=key.exists(){return Err("Incomplete PC identity. Restore its matching certificate/key, or explicitly reset both and pair again.".into());}
        if !cert.exists() {
            let temp=root.join(format!("identity-{}",crate::pairing::random_hex()?));std::fs::create_dir(&temp).map_err(|_|"Cannot initialize private PC identity")?;crate::trust::private_directory(&temp)?;
            let generated=(||->Result<(),String>{
                command(&["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", temp.join("key.pem").to_str().unwrap(), "-out", temp.join("cert.pem").to_str().unwrap(), "-days", "3650", "-subj", "/CN=Relay"])?;
                command(&["x509", "-in", temp.join("cert.pem").to_str().unwrap(), "-outform", "DER", "-out", temp.join("cert.der").to_str().unwrap()])?;
                command(&["pkcs8", "-topk8", "-nocrypt", "-in", temp.join("key.pem").to_str().unwrap(), "-outform", "DER", "-out", temp.join("key.der").to_str().unwrap()])?;
                crate::trust::protect_file(&temp.join("key.der"))?;std::fs::rename(temp.join("key.der"),&key).map_err(|_|"Cannot save PC key")?;std::fs::rename(temp.join("cert.der"),&cert).map_err(|_|"Cannot save PC certificate")?;Ok(())
            })();let _=std::fs::remove_dir_all(temp);generated?;
        }
        crate::trust::protect_file(&key)?;crate::trust::protect_file(&cert)?;
        for legacy in [root.join("key.pem"),root.join("cert.pem")]{if legacy.exists(){crate::trust::protect_file(&legacy)?;}}
        let digest = command(&["dgst", "-sha256", "-binary", cert.to_str().unwrap()])?;
        let pin: String = digest.iter().map(|v|format!("{v:02x}")).collect();
        let trusted=Arc::new(Mutex::new(crate::trust::Store::open(&root,&pin)?));
        let config = ServerConfig::builder().with_no_client_auth().with_single_cert(vec![CertificateDer::from(std::fs::read(cert).map_err(|e|e.to_string())?)], PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(std::fs::read(key).map_err(|e|e.to_string())?))).map_err(|e|e.to_string())?;
        let listener = TcpListener::bind("0.0.0.0:0").map_err(|e|e.to_string())?;
        let port = listener.local_addr().unwrap().port(); listener.set_nonblocking(true).map_err(|e|e.to_string())?;
        let udp = std::net::UdpSocket::bind("0.0.0.0:0").map_err(|e|e.to_string())?;
        udp.connect("192.0.2.1:80").map_err(|e|e.to_string())?;
        let host = udp.local_addr().map_err(|e|e.to_string())?.ip();
        let wireless_profile=wireless_profile.min(2);
        let endpoint=format!("{host}:{port}");
        let pairing=Arc::new(Mutex::new(crate::pairing::Pairing::new(Instant::now())?));
        let name=crate::profile::device_name(&name);
        use std::os::unix::process::CommandExt;
        let mut announce=Command::new("avahi-publish-service");announce.args([name.trim(),"_relay._tcp",&port.to_string(),"version=6","trust=p256-v1",&format!("id={pin}"),"audio=pcm48","watch=1",&format!("quality={wireless_profile}")]).stdout(Stdio::null()).stderr(Stdio::null());
        let owner=unsafe{libc::getpid()};
        // Normal shutdown is handled by Drop; this also withdraws discovery if the GUI is killed.
        unsafe{announce.pre_exec(move||{if libc::prctl(libc::PR_SET_PDEATHSIG,libc::SIGTERM)<0{return Err(std::io::Error::last_os_error());}if libc::getppid()!=owner{let _=libc::kill(libc::getpid(),libc::SIGTERM);}Ok(())});}
        let advertisement=announce.spawn().ok();
        let discovery=advertisement.is_some();
        let stop = Arc::new(AtomicBool::new(false)); let active = Arc::new(Mutex::new(None));
        let status = Arc::new(Mutex::new("Open Relay on your phone. Choose this computer under Nearby computers.".into()));
        let peer_name=Arc::new(Mutex::new("Phone".into()));let peer=peer_name.clone();
        let desktop=Arc::new(crate::pc_share::DesktopShare::default());let desktop_control=desktop.clone();
        let audio=Arc::new(crate::audio::AudioControl::default());let audio_control=audio.clone();
        let live=Arc::new(AtomicBool::new(false));let streaming=live.clone();
        let frame = Arc::new(Mutex::new(None));let video = frame.clone();
        let quit = stop.clone(); let state = status.clone(); let active_worker = active.clone();
        let pairing_worker=pairing.clone();let trust_worker=trusted.clone();let worker_pin=pin.clone();
        let active_peer=Arc::new(Mutex::new(None));let worker_peer=active_peer.clone();
        std::thread::spawn(move || {
            let config = Arc::new(config);
            while !quit.load(Ordering::Relaxed) {
                let Ok((socket, _)) = listener.accept() else { std::thread::sleep(Duration::from_millis(100)); continue; };
                let _ = socket.set_read_timeout(Some(Duration::from_secs(5))); let _ = socket.set_write_timeout(Some(Duration::from_secs(5))); let _ = socket.set_nodelay(true);
                *active_worker.lock().unwrap() = socket.try_clone().ok();
                let mut authenticated=false;
                let mut paired=false;
                let result = (|| -> Result<(), String> {
                    let conn = ServerConnection::new(config.clone()).map_err(|e|e.to_string())?;
                    let mut tls = StreamOwned::new(conn, socket);
                    let request=crate::pairing::authenticate_with_trust(&mut tls,&pairing_worker,Some(&trust_worker),&worker_pin,|request|{
                        let id=request.public_key.as_deref().map(crate::trust::peer_id).transpose().ok().flatten();
                        let approval={let mut pairing=pairing_worker.lock().unwrap();let approval=pairing.pending.clone().filter(|a|a.verification_code==request.verification_code&&a.device_id==id).unwrap_or_else(||Arc::new(crate::pairing::Approval{name:request.name.clone(),watching:request.mode=="view",pairing:request.mode=="pair",remember:request.remember,device_id:id,verification_code:request.verification_code.clone(),decision:std::sync::atomic::AtomicU8::new(0),expires:Instant::now()+crate::pairing::APPROVAL_TTL}));pairing.pending=Some(approval.clone());approval};ctx.request_repaint();
                        while !quit.load(Ordering::Relaxed)&&Instant::now()<approval.expires&&approval.decision.load(Ordering::SeqCst)==0{std::thread::sleep(Duration::from_millis(50));}
                        {let mut pairing=pairing_worker.lock().unwrap();if pairing.pending.as_ref().is_some_and(|pending|Arc::ptr_eq(pending,&approval)){pairing.pending=None;}}ctx.request_repaint();
                        approval.decision.load(Ordering::SeqCst)==1&&Instant::now()<approval.expires&&!quit.load(Ordering::Relaxed)
                    },Instant::now)?;
                    {let store=trust_worker.lock().unwrap();if request.generation!=Some(store.generation){return Err("Trust changed; pair again".into());}*worker_peer.lock().unwrap()=request.public_key.as_deref().map(crate::trust::peer_id).transpose()?;}
                    authenticated=true;*peer.lock().unwrap()=request.name.clone();
                    if request.mode=="pair"{paired=true;return Ok(());}
                    if request.mode=="view"{return crate::pc_share::serve(&mut tls,desktop_control.clone(),state.clone(),ctx.clone(),crate::profile::get(request.quality.min(wireless_profile)));}
                    *state.lock().unwrap() = "Sharing your Android screen · encrypted H.264 · stop on your phone or in Relay".into();
                    use ffmpeg_next as ff;
                    let mut decoder=video_decoder()?;
                    let mut audio_output=None;let mut audio_failed=false;
                    audio_control.received_frames.store(0,Ordering::Relaxed);audio_control.audible_frames.store(0,Ordering::Relaxed);audio_control.dropped_frames.store(0,Ordering::Relaxed);*audio_control.error.lock().unwrap()=None;
                    let mut scaler = None;
                    let mut decoded = ff::util::frame::video::Video::empty();
                    let mut rgba = ff::util::frame::video::Video::empty();
                    let mut packet_bytes = Vec::with_capacity(256*1024);
                    let mut frames = 0u64;let began=Instant::now();
                    loop {
                        if quit.load(Ordering::Relaxed){break;}
                        let kind=read_packet(&mut tls,&mut packet_bytes)?;if kind==PacketKind::Info{update_peer(&peer,&packet_bytes)?;continue;}if kind==PacketKind::Audio {
                            if audio_output.is_none()&&!audio_failed {match crate::audio::AudioOutput::open(audio_control.clone()){Ok(output)=>audio_output=Some(output),Err(error)=>{eprintln!("Relay: device audio unavailable: {error}");*audio_control.error.lock().unwrap()=Some(error);audio_failed=true;}}}
                            if let Some(output)=&mut audio_output{output.push(&packet_bytes);}
                            continue;
                        }
                        decoder.send_packet(&ff::Packet::copy(&packet_bytes)).map_err(|e|e.to_string())?;
                        while decoder.receive_frame(&mut decoded).is_ok(){
                            if decoded.width()>4096||decoded.height()>4096{return Err("Video dimensions too large".into());}
                            let key=(decoded.format(),decoded.width(),decoded.height());
                            if scaler.as_ref().is_none_or(|(k,_)|*k!=key){
                                scaler=Some((key,ff::software::scaling::context::Context::get(key.0,key.1,key.2,ff::format::Pixel::RGBA,key.1,key.2,ff::software::scaling::flag::Flags::FAST_BILINEAR).map_err(|e|e.to_string())?));
                                rgba=ff::util::frame::video::Video::empty();
                            }
                            scaler.as_mut().unwrap().1.run(&decoded,&mut rgba).map_err(|e|e.to_string())?;
                            let width=rgba.width() as usize;let height=rgba.height() as usize;
                            let pixels=rgba.data(0).chunks(rgba.stride(0)).take(height).flat_map(|row|row[..width*4].chunks_exact(4).map(|pixel|egui::Color32::from_rgb(pixel[0],pixel[1],pixel[2]))).collect();
                            *video.lock().unwrap()=Some(Arc::new(egui::ColorImage::new([width,height],pixels)));
                            frames+=1;streaming.store(true,Ordering::Relaxed);ctx.request_repaint();
                            if frames==1{eprintln!("Relay: first decoded frame in {} ms",began.elapsed().as_millis());}
                            if frames==1||frames%30==0{*state.lock().unwrap()=format!("Live · {} × {} · {:.0} FPS · encrypted",width,height,frames as f64/began.elapsed().as_secs_f64().max(0.001));}
                        }
                    }
                    Ok(())
                })();
                streaming.store(false,Ordering::Relaxed);ctx.request_repaint();
                *active_worker.lock().unwrap() = None;*worker_peer.lock().unwrap()=None;
                if !authenticated{if let Some(pending)=pairing_worker.lock().unwrap().pending.take(){pending.decide(false);ctx.request_repaint();}continue;}
                *state.lock().unwrap() = if paired{format!("Paired with {}. Choose it on your phone to start mirroring.",peer.lock().unwrap())}else{match result { Ok(()) => "Sharing ended. You can reconnect while this code is open.".into(), Err(e) => format!("Connection ended: {e}") }};
            }
            
        });
        Ok(Self { peer_name,desktop,audio,computer_name:name,advertisement, discovery, pairing, trusted, active_peer, endpoint, pin, quality:wireless_profile, status, frame, live, stop, active })
    }
}
impl Drop for Companion { fn drop(&mut self) {self.desktop.stop();if let Some(mut child)=self.advertisement.take(){let _=child.kill();let _=child.wait();}self.stop.store(true, Ordering::Relaxed); if let Some(s)=self.active.lock().unwrap().take(){let _=s.shutdown(std::net::Shutdown::Both);} } }

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn rejects_oversize_and_truncated_packets() {
        for size in [0u32,4*1024*1024+1] {assert!(read_packet(&mut &size.to_be_bytes()[..],&mut vec![]).is_err());}
        assert!(read_packet(&mut &b"\0\0\0\x04ab"[..],&mut vec![]).is_err());
        let mut bytes=vec![];read_packet(&mut &b"\0\0\0\x04abcd"[..],&mut bytes).unwrap();assert_eq!(bytes,b"abcd");
    }
    #[test] fn audio_and_video_packets_are_distinguished_and_audio_is_aligned() {
        let mut bytes=vec![];
        assert_eq!(read_packet(&mut &b"\x80\0\0\x04abcd"[..],&mut bytes).unwrap(),PacketKind::Audio);
        assert_eq!(bytes,b"abcd");
        assert_eq!(read_packet(&mut &b"\0\0\0\x04abcd"[..],&mut bytes).unwrap(),PacketKind::Video);
        assert!(read_packet(&mut &b"\x80\0\0\x03abc"[..],&mut bytes).is_err());
    }
    #[test] fn first_keyframe_decodes_without_probe_or_playback_buffer() {
        let mut decoder=video_decoder().unwrap();
        let packet=ffmpeg_next::Packet::copy(include_bytes!("../tests/fixtures/keyframe.h264"));
        decoder.send_packet(&packet).unwrap();
        let mut frame=ffmpeg_next::util::frame::video::Video::empty();
        decoder.receive_frame(&mut frame).expect("First packet must display immediately without waiting for another frame");
        assert_eq!((frame.width(),frame.height()),(32,32));
    }
}
