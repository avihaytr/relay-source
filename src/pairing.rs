// SPDX-License-Identifier: MPL-2.0
//! Protocol v6 QR authorization plus opt-in, signed trusted-device reconnects.
use std::{io::{Read, Write}, sync::{Arc, Mutex, atomic::{AtomicU8, Ordering}}, time::{Duration, Instant}};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
pub const QR_TTL: Duration = Duration::from_secs(120);
pub const SESSION_TTL: Duration = Duration::from_secs(10);
pub const PIN_CHALLENGE_TTL: Duration = Duration::from_secs(30);
pub const APPROVAL_TTL: Duration = Duration::from_secs(45);

// Deliberately no Debug/Serialize: bearer credentials must not reach diagnostics.
pub fn random_hex()->Result<String,String>{let mut bytes=[0u8;32];std::fs::File::open("/dev/urandom").and_then(|mut f|f.read_exact(&mut bytes)).map_err(|_|"Secure random source unavailable")?;Ok(crate::trust::hex(&bytes))}
pub struct Ticket { secret: String, pin:String, pin_attempts:u8, expires: Instant, used: bool }
impl Ticket {
    pub fn new(now: Instant, ttl: Duration) -> Result<Self, String> {
        let mut bytes=[0u8;32]; std::fs::File::open("/dev/urandom").and_then(|mut f|f.read_exact(&mut bytes)).map_err(|_|"Secure random source unavailable")?;
        let pin=u32::from_be_bytes(bytes[..4].try_into().unwrap())%1_000_000;
        Ok(Self{secret:bytes.iter().map(|b|format!("{b:02x}")).collect(),pin:format!("{pin:06}"),pin_attempts:0,expires:now+ttl,used:false})
    }
    fn available(&self, now:Instant)->bool{!self.used&&now<self.expires}
    fn redeem(&mut self, value:&str, now:Instant)->bool{
        if !self.available(now){return false;}
        if value.len()==64&&bool::from(self.secret.as_bytes().ct_eq(value.as_bytes())){self.used=true;return true;}
        if value.len()==6&&value.bytes().all(|b|b.is_ascii_digit()){
            if bool::from(self.pin.as_bytes().ct_eq(value.as_bytes())){self.used=true;return true;}
            self.pin_attempts=self.pin_attempts.saturating_add(1);if self.pin_attempts>=5{self.used=true;}
        }
        false
    }
}
pub struct Approval {pub name:String,pub watching:bool,pub pairing:bool,pub remember:bool,pub device_id:Option<String>,pub verification_code:Option<String>,pub decision:AtomicU8,pub expires:Instant}
impl Approval {pub fn decide(&self,allow:bool){let _=self.decision.compare_exchange(0,if allow{1}else{2},Ordering::SeqCst,Ordering::SeqCst);}}
pub struct Pairing {ticket:Ticket,pub pending:Option<Arc<Approval>>}
impl Pairing {
    pub fn new(now:Instant)->Result<Self,String>{Ok(Self{ticket:Ticket::new(now,QR_TTL)?,pending:None})}
    pub fn qr_code(&mut self,now:Instant)->Result<&str,String>{if !self.ticket.available(now){self.ticket=Ticket::new(now,QR_TTL)?;}Ok(&self.ticket.secret)}
    pub fn pin_code(&mut self,now:Instant)->Result<&str,String>{if !self.ticket.available(now){self.ticket=Ticket::new(now,QR_TTL)?;}Ok(&self.ticket.pin)}
    pub fn refresh(&mut self)->Result<(),String>{if let Some(pending)=&self.pending{pending.decide(false);}self.ticket=Ticket::new(Instant::now(),QR_TTL)?;Ok(())}
    fn redeem(&mut self,code:&str,now:Instant)->bool{self.ticket.redeem(code,now)}
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {pub version:u32, #[serde(default)]pub code:String, pub name:String, pub mode:String, pub quality:u32,
    #[serde(default)]pub public_key:Option<String>,#[serde(default)]pub auth:String,#[serde(default)]pub remember:bool,
    #[serde(skip)]pub remembered:bool,#[serde(skip)]pub generation:Option<u64>,#[serde(skip)]pub verification_code:Option<String>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]struct SignatureProof{signature:String}
pub fn transcript(pin:&str,nonce:&str,r:&Request)->String{format!("Relay-v6-trust1\n{pin}\n{nonce}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",r.public_key.as_deref().unwrap_or(""),r.mode,r.quality,crate::trust::hex(r.name.as_bytes()),match r.auth.as_str(){"trusted"=>"trusted","pin"=>"pin",_=>"qr"},r.code,if r.remember{"1"}else{"0"})}
pub fn verification_code(pin:&str,nonce:&str,public:&str)->String{let input=format!("Relay-v6-pin-check\n{pin}\n{nonce}\n{public}");let digest=ring::digest::digest(&ring::digest::SHA256,input.as_bytes());let value=u32::from_be_bytes(digest.as_ref()[..4].try_into().unwrap())%1_000_000;format!("{value:06}")}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionProof {session:String}
pub fn read_json<T:serde::de::DeserializeOwned>(io:&mut impl Read)->Result<T,String>{
    let mut size=[0;4];io.read_exact(&mut size).map_err(|_|"Connection closed")?;let size=u32::from_be_bytes(size)as usize;
    if size==0||size>1024{return Err("Invalid authentication message".into());}
    let mut bytes=vec![0;size];io.read_exact(&mut bytes).map_err(|_|"Connection closed")?;
    serde_json::from_slice(&bytes).map_err(|_|"Invalid authentication message".into())
}
pub fn write_json(io:&mut impl Write,value:&impl Serialize)->Result<(),String>{let bytes=serde_json::to_vec(value).map_err(|_|"Authentication encoding failed")?;io.write_all(&(bytes.len()as u32).to_be_bytes()).and_then(|_|io.write_all(&bytes)).and_then(|_|io.flush()).map_err(|_|"Connection closed".into())}
/// Original v6 QR flow remains available for session-only clients. Trusted
/// reconnects additionally require a saved public key and a fresh signed nonce.
#[cfg(test)]
pub fn authenticate<S:Read+Write>(io:&mut S,state:&Mutex<Pairing>,approve:impl FnOnce(&Request)->bool,now:impl FnMut()->Instant)->Result<Request,String>{authenticate_with_trust(io,state,None,"",approve,now)}
pub fn authenticate_with_trust<S:Read+Write>(io:&mut S,state:&Mutex<Pairing>,store:Option<&Mutex<crate::trust::Store>>,pin:&str,approve:impl FnOnce(&Request)->bool,mut now:impl FnMut()->Instant)->Result<Request,String>{
    let mut magic=[0;4];io.read_exact(&mut magic).map_err(|_|"Connection closed")?;
    if &magic!=b"RLY6"{return Err("Unsupported pairing protocol".into());}
    let mut request:Request=read_json(io)?;
    if request.version!=6||!matches!(request.mode.as_str(),"send"|"view"|"pair")||request.quality>2||request.name.is_empty()||request.name.len()>50||request.name.chars().any(|c|c.is_control())||!matches!(request.auth.as_str(),""|"qr"|"trusted"|"pin") {return Err("Invalid authentication request".into());}
    let trusted=request.auth=="trusted";
    let id=request.public_key.as_deref().map(crate::trust::peer_id).transpose()?;
    let (generation,require_approval)=if let Some(store)=store{let store=store.lock().unwrap();if store.server_pin!=pin{return Err("Computer identity changed".into());}if trusted{let id=id.as_ref().ok_or("Missing device identity")?;let peer=store.peers.get(id).ok_or("Device trust revoked or unknown; scan again")?;if Some(&peer.public_key)!=request.public_key.as_ref(){return Err("Device identity changed".into());}}(Some(store.generation),store.require_approval)}else{if trusted||request.public_key.is_some()||request.remember{return Err("Persistent identity unavailable".into());}(None,true)};
    if trusted{if !request.code.is_empty()||request.remember{return Err("Invalid remembered-device request".into());}}
    else if request.auth=="pin"{if request.mode!="pair"||request.code.len()!=6||!request.code.bytes().all(|b|b.is_ascii_digit())||!state.lock().unwrap().redeem(&request.code,now()){return Err("Pairing PIN is incorrect, expired, or already used.".into());}}
    else if request.code.len()!=64||!request.code.bytes().all(|b|b.is_ascii_hexdigit())||!state.lock().unwrap().redeem(&request.code,now()){return Err("Pairing rejected or expired. Scan a fresh QR code.".into());}
    if request.remember&&id.is_none(){return Err("Remembering requires proof of device identity".into());}
    if request.mode=="pair"&&(!request.remember||trusted||request.public_key.is_none()||!matches!(request.auth.as_str(),"qr"|"pin")){return Err("Pairing requires a new signed device identity and explicit enrollment".into());}
    if let Some(public)=&request.public_key{
        let challenge_ttl=if request.auth=="pin"{PIN_CHALLENGE_TTL}else{SESSION_TTL};let mut challenge=Ticket::new(now(),challenge_ttl)?;let nonce=challenge.secret.clone();
        if request.auth=="pin"{
            let code=verification_code(pin,&nonce,public);request.verification_code=Some(code.clone());
            let approval=Arc::new(Approval{name:request.name.clone(),watching:false,pairing:true,remember:true,device_id:id.clone(),verification_code:Some(code),decision:AtomicU8::new(0),expires:now()+APPROVAL_TTL});
            state.lock().unwrap().pending=Some(approval);
        }
        let mut challenge_message=serde_json::json!({"status":"challenge","nonce":nonce,"expiresIn":challenge_ttl.as_secs()});
        if let Some(code)=&request.verification_code{challenge_message["verifyCode"]=serde_json::Value::String(code.clone());}
        write_json(io,&challenge_message)?;
        let proof:SignatureProof=read_json(io)?;
        if !challenge.redeem(&nonce,now()){return Err("Device challenge expired".into());}
        crate::trust::verify(public,&transcript(pin,&nonce,&request),&proof.signature)?;
    }
    if !trusted||require_approval {let approved=approve(&request);if let Some(pending)=state.lock().unwrap().pending.as_ref(){pending.decide(false);}state.lock().unwrap().pending=None;if !approved{return Err("Pairing was not approved".into());}}
    if let Some(store)=store{if Some(store.lock().unwrap().generation)!=generation{return Err("Device trust changed during authentication".into());}}
    let mut session=Ticket::new(now(),SESSION_TTL)?;
    write_json(io,&serde_json::json!({"status":"approved","session":session.secret,"expiresIn":SESSION_TTL.as_secs(),"remembered":trusted||request.remember}))?;
    let proof:SessionProof=read_json(io)?;
    if !session.redeem(&proof.session,now()){return Err("Session authorization rejected or expired".into());}
    if let Some(store)=store{let mut store=store.lock().unwrap();if Some(store.generation)!=generation{return Err("Device trust revoked during authentication".into());}if request.remember{store.remember(&request.name,request.public_key.as_deref().ok_or("Missing identity")?)?;}request.generation=Some(store.generation);}
    request.remembered=trusted||request.remember;
    io.write_all(b"OK").and_then(|_|io.flush()).map_err(|_|"Connection closed")?;
    Ok(request)
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn ticket_expiration_single_use_and_constant_time_value_check(){let now=Instant::now();let mut t=Ticket::new(now,SESSION_TTL).unwrap();let code=t.secret.clone();assert!(!t.redeem(&"0".repeat(64),now));assert!(t.redeem(&code,now));assert!(!t.redeem(&code,now));let mut t=Ticket::new(now,SESSION_TTL).unwrap();let code=t.secret.clone();assert!(!t.redeem(&code,now+SESSION_TTL));}
    #[test] fn pin_verification_code_has_a_stable_cross_language_vector(){assert_eq!(verification_code(&"1".repeat(64),&"2".repeat(64),&format!("04{}", "a".repeat(128))),"766134");}
    #[test] fn pin_is_six_digits_single_use_and_locks_after_five_wrong_attempts(){let now=Instant::now();let mut t=Ticket::new(now,QR_TTL).unwrap();let pin=t.pin.clone();assert_eq!(pin.len(),6);let mut wrong=pin.clone();let last=wrong.pop().unwrap();wrong.push(if last=='9'{'8'}else{char::from_u32(last as u32+1).unwrap()});for _ in 0..4{assert!(!t.redeem(&wrong,now));}assert!(t.redeem(&pin,now));assert!(!t.redeem(&pin,now));let mut t=Ticket::new(now,QR_TTL).unwrap();let pin=t.pin.clone();for _ in 0..5{assert!(!t.redeem(&wrong,now));}assert!(!t.redeem(&pin,now));}
    #[test] fn reconnect_requires_new_qr_and_old_qr_stays_invalid(){let now=Instant::now();let mut p=Pairing::new(now).unwrap();let old=p.qr_code(now).unwrap().to_string();assert!(p.redeem(&old,now));let fresh=p.qr_code(now).unwrap().to_string();assert_ne!(old,fresh);assert!(!p.redeem(&old,now));assert!(p.redeem(&fresh,now));}
    #[test] fn expired_qr_is_rejected_and_refresh_invalidates_old(){let now=Instant::now();let mut p=Pairing::new(now).unwrap();let old=p.qr_code(now).unwrap().to_string();assert!(!p.redeem(&old,now+QR_TTL));let fresh=p.qr_code(now+QR_TTL).unwrap().to_string();assert_ne!(old,fresh);p.refresh().unwrap();assert!(!p.redeem(&fresh,now));}
    fn exchange(mode:&str,approve:bool,expired_qr:bool,expired_session:bool,wrong_proof:bool)->Result<Request,String>{
        use std::os::unix::net::UnixStream;
        let now=Instant::now();let state=Mutex::new(Pairing::new(now).unwrap());let code=state.lock().unwrap().qr_code(now).unwrap().to_string();let(mut server,mut client)=UnixStream::pair().unwrap();server.set_read_timeout(Some(Duration::from_secs(2))).unwrap();client.set_read_timeout(Some(Duration::from_secs(2))).unwrap();let mode=mode.to_string();
        let worker=std::thread::spawn(move||{let mut calls=0;authenticate(&mut server,&state,|_|approve,||{calls+=1;if expired_qr{now+QR_TTL}else if expired_session&&calls==3{now+SESSION_TTL}else{now}})});
        client.write_all(b"RLY6").unwrap();write_json(&mut client,&serde_json::json!({"version":6,"code":code,"name":"Test phone","mode":mode,"quality":2})).unwrap();
        if let Ok(response)=read_json::<serde_json::Value>(&mut client){let proof=if wrong_proof{"0".repeat(64)}else{response["session"].as_str().unwrap().to_string()};write_json(&mut client,&serde_json::json!({"session":proof})).unwrap();let mut ack=[0;2];let _=client.read_exact(&mut ack);}
        worker.join().unwrap()
    }
    #[test] fn both_directions_authenticate_only_after_approval(){assert_eq!(exchange("send",true,false,false,false).unwrap().mode,"send");assert_eq!(exchange("view",true,false,false,false).unwrap().mode,"view");}
    #[test] fn explicit_rejection_expired_credentials_and_wrong_proof_fail(){assert!(exchange("send",false,false,false,false).is_err());assert!(exchange("send",true,true,false,false).is_err());assert!(exchange("send",true,false,true,false).is_err());assert!(exchange("send",true,false,false,true).is_err());assert!(exchange("unknown",true,false,false,false).is_err());}
    #[test] fn legacy_auth_and_oversized_requests_fail_closed(){let state=Mutex::new(Pairing::new(Instant::now()).unwrap());let mut legacy=std::io::Cursor::new(b"RELAY token".to_vec());assert!(authenticate(&mut legacy,&state,|_|panic!("Must not ask for approval"),Instant::now).is_err());assert!(read_json::<Request>(&mut &2048u32.to_be_bytes()[..]).is_err());}
    #[test] fn authentication_over_real_tls_rejects_replay_and_allows_fresh_reconnection(){
        use rustls::{ServerConfig,ServerConnection,ClientConfig,ClientConnection,RootCertStore,StreamOwned,pki_types::{CertificateDer,PrivateKeyDer,PrivatePkcs8KeyDer,ServerName}};
        use std::{net::{TcpListener,TcpStream},process::Command,os::unix::fs::PermissionsExt};
        struct Temp(std::path::PathBuf);impl Drop for Temp{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let temp=Temp(std::env::temp_dir().join(format!("relay-auth-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())));
        std::fs::create_dir(&temp.0).unwrap();std::fs::set_permissions(&temp.0,std::fs::Permissions::from_mode(0o700)).unwrap();
        let key=temp.0.join("key.pem");let pem=temp.0.join("cert.pem");let cert=temp.0.join("cert.der");let derkey=temp.0.join("key.der");
        let generated=Command::new("openssl").args(["req","-x509","-newkey","rsa:2048","-nodes","-keyout",key.to_str().unwrap(),"-out",pem.to_str().unwrap(),"-days","1","-subj","/CN=localhost","-addext","subjectAltName=DNS:localhost","-addext","basicConstraints=critical,CA:FALSE"]).output().unwrap();assert!(generated.status.success());
        assert!(Command::new("openssl").args(["x509","-in",pem.to_str().unwrap(),"-outform","DER","-out",cert.to_str().unwrap()]).output().unwrap().status.success());
        assert!(Command::new("openssl").args(["pkcs8","-topk8","-nocrypt","-in",key.to_str().unwrap(),"-outform","DER","-out",derkey.to_str().unwrap()]).output().unwrap().status.success());
        let certificate=CertificateDer::from(std::fs::read(cert).unwrap());let server=Arc::new(ServerConfig::builder().with_no_client_auth().with_single_cert(vec![certificate.clone()],PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(std::fs::read(derkey).unwrap()))).unwrap());
        let mut roots=RootCertStore::empty();roots.add(certificate).unwrap();let client=Arc::new(ClientConfig::builder().with_root_certificates(roots).with_no_client_auth());
        let state=Arc::new(Mutex::new(Pairing::new(Instant::now()).unwrap()));
        let old=state.lock().unwrap().qr_code(Instant::now()).unwrap().to_string();
        // Pair once, reject replay, reject an explicit user denial, then reconnect
        // with a new QR. The server sees no media until the final OK.
        let mut previous_session=String::new();
        for step in 0..5 {
            let (code,approval,expected)=match step{0=>(old.clone(),true,true),1=>(old.clone(),false,false),2=>(state.lock().unwrap().qr_code(Instant::now()).unwrap().to_string(),false,false),3=>(state.lock().unwrap().qr_code(Instant::now()).unwrap().to_string(),true,true),_=>(state.lock().unwrap().qr_code(Instant::now()).unwrap().to_string(),true,false)};
            let listener=TcpListener::bind("127.0.0.1:0").unwrap();let address=listener.local_addr().unwrap();let config=server.clone();let shared=state.clone();
            let worker=std::thread::spawn(move||{let(socket,_)=listener.accept().unwrap();socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();let mut tls=StreamOwned::new(ServerConnection::new(config).unwrap(),socket);authenticate(&mut tls,&shared,|_|approval,Instant::now).is_ok()});
            let socket=TcpStream::connect(address).unwrap();socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();let mut tls=StreamOwned::new(ClientConnection::new(client.clone(),ServerName::try_from("localhost").unwrap().to_owned()).unwrap(),socket);
            tls.write_all(b"RLY6").unwrap();write_json(&mut tls,&serde_json::json!({"version":6,"code":code,"name":"Test phone","mode":"send","quality":1})).unwrap();
            let result=read_json::<serde_json::Value>(&mut tls);if let Ok(response)=result{let current=response["session"].as_str().unwrap().to_string();if step==0{previous_session=current.clone();}let proof=if step==4{previous_session.clone()}else{current};write_json(&mut tls,&serde_json::json!({"session":proof})).unwrap();let mut ok=[0;2];let result=tls.read_exact(&mut ok);if expected{result.unwrap();assert_eq!(&ok,b"OK");}else{assert!(result.is_err());}}else{assert!(!expected);}
            assert_eq!(worker.join().unwrap(),expected);
        }
        // A TLS peer without the pinned trust anchor fails before pairing.
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();let address=listener.local_addr().unwrap();let config=server.clone();let shared=state.clone();
        let worker=std::thread::spawn(move||{let(socket,_)=listener.accept().unwrap();socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();let mut tls=StreamOwned::new(ServerConnection::new(config).unwrap(),socket);assert!(authenticate(&mut tls,&shared,|_|panic!("Unverified TLS must not reach approval"),Instant::now).is_err());});
        let untrusted=Arc::new(ClientConfig::builder().with_root_certificates(RootCertStore::empty()).with_no_client_auth());let socket=TcpStream::connect(address).unwrap();socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();let mut tls=StreamOwned::new(ClientConnection::new(untrusted,ServerName::try_from("localhost").unwrap().to_owned()).unwrap(),socket);assert!(tls.write_all(b"RLY6").is_err());drop(tls);worker.join().unwrap();
    }

}
