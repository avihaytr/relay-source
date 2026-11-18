// SPDX-License-Identifier: MPL-2.0
//! View-only desktop sharing. The portal supplies a permission-scoped PipeWire FD.
use std::{io::{Read,Write},os::{fd::AsRawFd,unix::process::CommandExt},process::{Child,Command,Stdio},sync::{Arc,Mutex,atomic::{AtomicBool,AtomicU64,Ordering}},time::Duration};
use ashpd::desktop::{Session,PersistMode,screencast::{Screencast,SelectSourcesOptions,SourceType,CursorMode}};

#[derive(Default)]
pub struct DesktopShare {
    pub pending:AtomicBool,
    pub active:AtomicBool,
    pub cancel:AtomicBool,
    pub frames:AtomicU64,
}
impl DesktopShare {pub fn stop(&self){self.cancel.store(true,Ordering::Relaxed);}}
// ashpd may retain D-Bus tasks beyond a single capture. Their executor must
// remain alive until process exit, including while no screen is being shared.
fn portal_runtime() -> Result<&'static tokio::runtime::Runtime, String> {
    static RUNTIME: std::sync::OnceLock<Result<tokio::runtime::Runtime, String>> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1).thread_name("relay-portal").enable_all().build()
        .map_err(|e| format!("Could not start screen-sharing runtime: {e}")))
        .as_ref().map_err(Clone::clone)
}
const SETUP_TIMEOUT: Duration = Duration::from_secs(10);
const PICKER_TIMEOUT: Duration = Duration::from_secs(120);
const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

async fn portal_call<T>(cancel: &AtomicBool, timeout: Duration, phase: &str,
    operation: impl std::future::Future<Output=Result<T, String>>) -> Result<T, String> {
    tokio::select! {
        biased;
        _ = async { while !cancel.load(Ordering::Acquire) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }} => Err("Screen sharing was cancelled".into()),
        result = tokio::time::timeout(timeout, operation) =>
            result.map_err(|_| format!("Screen-sharing portal timed out during {phase}"))?,
    }
}

// A dedicated connection makes portal requests/session handles session-scoped.
// Closing it also releases a session created remotely just as creation timed out,
// before ashpd could return its handle. No persistent capture grant is requested.
fn session_already_closed(error: &ashpd::Error) -> bool {
    match error {
        ashpd::Error::Zbus(ashpd::zbus::Error::MethodError(name, ..)) |
        ashpd::Error::Portal(ashpd::PortalError::ZBus(ashpd::zbus::Error::MethodError(name, ..))) =>
            matches!(name.as_str(), "org.freedesktop.DBus.Error.UnknownObject" | "org.freedesktop.DBus.Error.UnknownMethod"),
        ashpd::Error::Zbus(ashpd::zbus::Error::FDO(error)) |
        ashpd::Error::Portal(ashpd::PortalError::ZBus(ashpd::zbus::Error::FDO(error))) =>
            matches!(error.as_ref(), ashpd::zbus::fdo::Error::UnknownObject(_) | ashpd::zbus::fdo::Error::UnknownMethod(_)),
        ashpd::Error::Portal(ashpd::PortalError::NotFound(_)) => true,
        _ => false,
    }
}
struct PortalGuard {
    runtime: &'static tokio::runtime::Runtime,
    connection: ashpd::zbus::Connection,
    session: Option<Session<Screencast>>,
}
impl Drop for PortalGuard {
    fn drop(&mut self) {
        self.runtime.block_on(async {
            if let Some(session) = &self.session {
                match tokio::time::timeout(CLOSE_TIMEOUT, session.close()).await {
                    Ok(Ok(())) => {},
                    // Cancellation may already have destroyed the portal session.
                    Ok(Err(error)) if session_already_closed(&error) => {},
                    _ => eprintln!("Relay: portal session close failed; closing its D-Bus connection"),
                }
            }
            match tokio::time::timeout(CLOSE_TIMEOUT, self.connection.clone().close()).await {
                Ok(Ok(())) => {},
                _ => eprintln!("Relay: portal connection close failed or timed out"),
            }
        });
    }
}

struct SessionGuard {
    state: Arc<DesktopShare>, ctx: egui::Context, done: Arc<AtomicBool>,
    watcher: Option<std::thread::JoinHandle<()>>,
}
impl Drop for SessionGuard {
    fn drop(&mut self) {
        self.done.store(true, Ordering::Release);
        self.state.stop();
        if let Some(watcher) = self.watcher.take() { let _ = watcher.join(); }
        self.state.pending.store(false, Ordering::Relaxed);
        self.state.active.store(false, Ordering::Relaxed);
        self.ctx.request_repaint();
    }
}
fn watch_connection(socket: std::net::TcpStream, state: Arc<DesktopShare>, done: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut byte = [0];
        while !done.load(Ordering::Acquire) && !state.cancel.load(Ordering::Relaxed) {
            match socket.peek(&mut byte) {
                Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {},
                _ => { if !done.load(Ordering::Acquire) { state.stop(); } break; },
            }
        }
    })
}

struct Capture {
    child: Arc<Mutex<Child>>, done: Arc<AtomicBool>,
    workers: Vec<std::thread::JoinHandle<()>>,
}
impl Drop for Capture {
    fn drop(&mut self) {
        self.done.store(true, Ordering::Release);
        { let mut child = self.child.lock().unwrap(); let _ = child.kill(); let _ = child.wait(); }
        // Kill/reap before joining pipe readers; release the child lock first.
        for worker in self.workers.drain(..) { let _ = worker.join(); }
    }
}

pub fn serve(tls:&mut rustls::StreamOwned<rustls::ServerConnection,std::net::TcpStream>,state:Arc<DesktopShare>,status:Arc<Mutex<String>>,ctx:egui::Context,profile:crate::profile::Profile)->Result<(),String>{
    state.cancel.store(false,Ordering::Relaxed);state.frames.store(0,Ordering::Relaxed);state.pending.store(true,Ordering::Relaxed);
    let done=Arc::new(AtomicBool::new(false));
    let mut guard=SessionGuard{state:state.clone(),ctx:ctx.clone(),done:done.clone(),watcher:None};
    *status.lock().unwrap()="Choose the screen or window to share in the desktop picker.".into();ctx.request_repaint();
    let socket=tls.sock.try_clone().map_err(|e|e.to_string())?;socket.set_read_timeout(Some(Duration::from_millis(250))).map_err(|e|e.to_string())?;
    guard.watcher=Some(watch_connection(socket,state.clone(),done));
    let runtime=portal_runtime()?;
    let connection=runtime.block_on(portal_call(&state.cancel,SETUP_TIMEOUT,"connection",async {
        ashpd::zbus::Connection::session().await.map_err(|e|e.to_string())
    }))?;
    let mut portal=PortalGuard{runtime,connection,session:None};
    let proxy=runtime.block_on(portal_call(&state.cancel,SETUP_TIMEOUT,"initialization",async {
        Screencast::with_connection(portal.connection.clone()).await.map_err(|e|e.to_string())
    }))?;
    portal.session=Some(runtime.block_on(portal_call(&state.cancel,SETUP_TIMEOUT,"session creation",async {
        proxy.create_session(Default::default()).await.map_err(|e|e.to_string())
    }))?);
    let session=portal.session.as_ref().unwrap();
    let selected=runtime.block_on(portal_call(&state.cancel,PICKER_TIMEOUT,"screen selection",async {
        proxy.select_sources(session,SelectSourcesOptions::default().set_cursor_mode(CursorMode::EMBEDDED).set_sources(SourceType::MONITOR|SourceType::WINDOW).set_multiple(false).set_persist_mode(PersistMode::DoNot)).await.map_err(|e|e.to_string())?.response().map_err(|e|e.to_string())?;
        proxy.start(session,None,Default::default()).await.map_err(|e|e.to_string())?.response().map_err(|e|e.to_string())
    }));
    let result=(||->Result<(),String>{
        let selection=selected.map_err(|e|format!("Screen sharing was not approved: {e}"))?;
        let stream=selection.streams().first().ok_or("No screen or window was selected")?;
        let node=stream.pipe_wire_node_id();let serial=stream.pipewire_serial().unwrap_or(0);
        let fd=runtime.block_on(portal_call(&state.cancel,SETUP_TIMEOUT,"PipeWire connection",async {
            proxy.open_pipe_wire_remote(session,Default::default()).await.map_err(|e|e.to_string())
        }))?;
        let raw_fd=fd.as_raw_fd();
        let mut command=Command::new("python3");command.args(["-u","-c",include_str!("../tools/desktop_capture.py"),&raw_fd.to_string(),&node.to_string(),&serial.to_string(),&profile.width.to_string(),&profile.height.to_string(),&profile.fps.to_string(),&profile.kbps.to_string()]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        // The portal FD is inherited only by this owned capture process.
        unsafe{command.pre_exec(move||{let flags=libc::fcntl(raw_fd,libc::F_GETFD);if flags<0{return Err(std::io::Error::last_os_error());}if libc::fcntl(raw_fd,libc::F_SETFD,flags&!libc::FD_CLOEXEC)<0{return Err(std::io::Error::last_os_error());}Ok(())});}
        let mut child=command.spawn().map_err(|e|format!("Could not start PipeWire capture: {e}"))?;
        let mut output=child.stdout.take().unwrap();let errors=child.stderr.take().unwrap();
        let error_log=Arc::new(Mutex::new(String::new()));let error_copy=error_log.clone();
        let error_worker=std::thread::spawn(move||{let mut text=String::new();let _=errors.take(32*1024).read_to_string(&mut text);if !text.trim().is_empty(){eprintln!("Relay: desktop capture: {}",text.trim());}*error_copy.lock().unwrap()=text;});
        let mut capture=Capture{child:Arc::new(Mutex::new(child)),done:Arc::new(AtomicBool::new(false)),workers:vec![error_worker]};
        let process=capture.child.clone();let done=capture.done.clone();let cancel=state.clone();let cancel_worker=std::thread::spawn(move||{while !done.load(Ordering::Acquire){if cancel.cancel.load(Ordering::Relaxed){let _=process.lock().unwrap().kill();break;}std::thread::sleep(Duration::from_millis(100));}});
        capture.workers.push(cancel_worker);
        let mut first=true;let mut data=Vec::new();
        while !state.cancel.load(Ordering::Relaxed){
            let mut header=[0;4];if let Err(error)=output.read_exact(&mut header){if state.cancel.load(Ordering::Relaxed){break;}return Err(format!("Screen capture stopped: {error}. {}",error_log.lock().unwrap().trim()));}
            let length=u32::from_be_bytes(header)as usize;if length==0||length>4*1024*1024{return Err("Invalid desktop video packet".into());}
            data.resize(length,0);output.read_exact(&mut data).map_err(|e|e.to_string())?;
            if first{tls.write_all(b"PCV1").map_err(|e|e.to_string())?;for value in [profile.width,profile.height,profile.fps]{tls.write_all(&value.to_be_bytes()).map_err(|e|e.to_string())?;}first=false;state.pending.store(false,Ordering::Relaxed);state.active.store(true,Ordering::Relaxed);*status.lock().unwrap()="Sharing this computer to the phone · view only".into();ctx.request_repaint();eprintln!("Relay: PC-to-phone capture started · {}x{} · {} FPS · {} kbps",profile.width,profile.height,profile.fps,profile.kbps);}
            tls.write_all(&header).and_then(|_|tls.write_all(&data)).and_then(|_|tls.flush()).map_err(|e|e.to_string())?;
            state.frames.fetch_add(1,Ordering::Relaxed);
        }
        Ok(())
    })();
    drop(portal);
    if let Err(error)=&result {
        if !state.active.load(Ordering::Relaxed){let message=error.as_bytes();let message=&message[..message.len().min(4096)];let _=tls.write_all(b"ERR!");let _=tls.write_all(&(message.len()as u32).to_be_bytes());let _=tls.write_all(message);let _=tls.flush();}
    }
    eprintln!("Relay: PC-to-phone sharing ended after {} frames",state.frames.load(Ordering::Relaxed));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    struct DropCount(Arc<AtomicUsize>);
    impl Drop for DropCount { fn drop(&mut self) { self.0.fetch_add(1,Ordering::SeqCst); } }

    #[test]
    fn runtime_survives_call_scopes_and_keeps_its_tasks_alive() {
        let first=portal_runtime().unwrap();
        let task=first.spawn(async { tokio::time::sleep(Duration::from_millis(20)).await; 42 });
        let second=std::thread::spawn(|| portal_runtime().unwrap()).join().unwrap();
        assert!(std::ptr::eq(first,second));
        assert_eq!(second.block_on(task).unwrap(),42);
    }

    #[test]
    fn portal_timeout_drops_pending_operation() {
        let drops=Arc::new(AtomicUsize::new(0));let count=drops.clone();
        let cancel=AtomicBool::new(false);
        let result=portal_runtime().unwrap().block_on(portal_call(&cancel,Duration::from_millis(20),"test",async move {
            let _owned=DropCount(count);
            std::future::pending::<Result<(),String>>().await
        }));
        assert!(result.unwrap_err().contains("timed out"));
        assert_eq!(drops.load(Ordering::SeqCst),1);
    }

    #[test]
    fn cancellation_drops_pending_operation_and_next_call_succeeds() {
        let drops=Arc::new(AtomicUsize::new(0));let count=drops.clone();
        let cancel=AtomicBool::new(false);let runtime=portal_runtime().unwrap();
        runtime.block_on(async {
            let operation=portal_call(&cancel,Duration::from_secs(2),"test",async move {
                let _owned=DropCount(count);
                std::future::pending::<Result<(),String>>().await
            });
            let stop=async {tokio::time::sleep(Duration::from_millis(20)).await;cancel.store(true,Ordering::Release);};
            let (result,())=tokio::join!(operation,stop);
            assert!(result.unwrap_err().contains("cancelled"));
        });
        assert_eq!(drops.load(Ordering::SeqCst),1);
        cancel.store(false,Ordering::Release);
        assert_eq!(runtime.block_on(portal_call(&cancel,Duration::from_secs(1),"next",async {Ok(7)})).unwrap(),7);
    }

    #[test]
    fn already_cancelled_request_does_not_start_a_portal_operation() {
        let cancel=AtomicBool::new(true);
        let result=portal_runtime().unwrap().block_on(portal_call(&cancel,Duration::from_secs(1),"test",async {
            panic!("Cancelled requests must not open a picker");
            #[allow(unreachable_code)] Ok(())
        }));
        assert!(result.unwrap_err().contains("cancelled"));
    }

    #[test]
    fn session_cleanup_joins_watcher_before_state_can_be_reused() {
        let listener=std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let client=std::net::TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (socket,_)=listener.accept().unwrap();
        socket.set_read_timeout(Some(Duration::from_millis(30))).unwrap();
        let state=Arc::new(DesktopShare::default());state.pending.store(true,Ordering::Relaxed);
        let done=Arc::new(AtomicBool::new(false));
        let guard=SessionGuard{state:state.clone(),ctx:egui::Context::default(),done:done.clone(),watcher:Some(watch_connection(socket,state.clone(),done.clone()))};
        drop(guard);
        assert!(done.load(Ordering::Acquire));assert!(!state.pending.load(Ordering::Relaxed));
        state.cancel.store(false,Ordering::Relaxed);state.pending.store(true,Ordering::Relaxed);
        drop(client);
        std::thread::sleep(Duration::from_millis(60));
        assert!(!state.cancel.load(Ordering::Relaxed),"An old watcher cancelled the next session");
    }

    #[test]
    fn capture_cleanup_reaps_child_and_joins_pipe_reader() {
        let mut child=Command::new("sh").args(["-c","exec sleep 30"]).stdin(Stdio::null()).stderr(Stdio::piped()).spawn().unwrap();
        let mut errors=child.stderr.take().unwrap();
        let child=Arc::new(Mutex::new(child));let finished=Arc::new(AtomicBool::new(false));let observed=finished.clone();
        let reader=std::thread::spawn(move||{let mut data=Vec::new();errors.read_to_end(&mut data).unwrap();observed.store(true,Ordering::Release);});
        let capture=Capture{child:child.clone(),done:Arc::new(AtomicBool::new(false)),workers:vec![reader]};
        drop(capture);
        assert!(child.lock().unwrap().try_wait().unwrap().is_some());
        assert!(finished.load(Ordering::Acquire));
    }

    #[test]
    fn cleanup_only_accepts_already_removed_session_errors() {
        assert!(session_already_closed(&ashpd::Error::Portal(ashpd::PortalError::NotFound("closed".into()))));
        assert!(!session_already_closed(&ashpd::Error::Portal(ashpd::PortalError::NotAllowed("denied".into()))));
        assert!(!session_already_closed(&ashpd::Error::NoResponse));
        let removed=ashpd::zbus::Error::FDO(Box::new(ashpd::zbus::fdo::Error::UnknownObject("closed".into())));
        assert!(session_already_closed(&ashpd::Error::Portal(ashpd::PortalError::ZBus(removed))));
        let denied=ashpd::zbus::Error::FDO(Box::new(ashpd::zbus::fdo::Error::AccessDenied("denied".into())));
        assert!(!session_already_closed(&ashpd::Error::Zbus(denied)));
    }

    #[test]
    #[ignore = "Requires a running desktop portal; creates/closes sessions but never selects or captures a screen"]
    fn repeated_real_portal_sessions_close_their_connections() {
        let runtime=portal_runtime().unwrap();let cancel=AtomicBool::new(false);
        let monitor=runtime.block_on(ashpd::zbus::Connection::session()).unwrap();
        for _ in 0..3 {
            let connection=runtime.block_on(portal_call(&cancel,SETUP_TIMEOUT,"test connection",async {
                ashpd::zbus::Connection::session().await.map_err(|e|e.to_string())
            })).unwrap();
            let observer=connection.clone();let name=connection.unique_name().unwrap().to_owned();
            let mut guard=PortalGuard{runtime,connection,session:None};
            let proxy=runtime.block_on(portal_call(&cancel,SETUP_TIMEOUT,"test initialization",async {
                Screencast::with_connection(guard.connection.clone()).await.map_err(|e|e.to_string())
            })).unwrap();
            guard.session=Some(runtime.block_on(portal_call(&cancel,SETUP_TIMEOUT,"test creation",async {
                proxy.create_session(Default::default()).await.map_err(|e|e.to_string())
            })).unwrap());
            drop(proxy);drop(guard);
            runtime.block_on(async {
                tokio::time::timeout(Duration::from_secs(1),observer.closed()).await.unwrap();
                let bus=ashpd::zbus::fdo::DBusProxy::new(&monitor).await.unwrap();
                tokio::time::timeout(Duration::from_secs(1),async {
                    while bus.name_has_owner(name.clone().into()).await.unwrap() {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }).await.unwrap();
            });
        }
        runtime.block_on(monitor.close()).unwrap();
    }
}
