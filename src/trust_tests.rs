// SPDX-License-Identifier: MPL-2.0
//! Signed reconnect tests use an in-memory transport and temporary private stores.
use super::*;
use crate::pairing::{self,Request,Pairing};
use std::{sync::{Arc,Mutex,atomic::{AtomicU64,AtomicUsize,Ordering}},time::{Instant,Duration},io::{Read,Write},os::unix::net::UnixStream};
use ring::{signature::{EcdsaKeyPair,KeyPair,ECDSA_P256_SHA256_ASN1_SIGNING},rand::SystemRandom};
struct Fixture{root:std::path::PathBuf,store:Arc<Mutex<Store>>,pair:Arc<Mutex<Pairing>>,key:EcdsaKeyPair,public:String,pin:String,now:Instant}
impl Drop for Fixture{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.root);}}
impl Fixture {
    fn new()->Self{let root=std::env::temp_dir().join(format!("relay-trust-test-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));let pin="1".repeat(64);let store=Arc::new(Mutex::new(Store::open(&root,&pin).unwrap()));let now=Instant::now();let key=make_key();let public=hex(key.public_key().as_ref());Self{root,store,pair:Arc::new(Mutex::new(Pairing::new(now).unwrap())),key,public,pin,now}}
    fn id(&self)->String{peer_id(&self.public).unwrap()}
    fn exchange(&self,auth:&str,remember:bool,approval:bool,attack:&str)->(Result<Request,String>,usize,Option<String>){self.exchange_mode(auth,remember,approval,attack,"send")}
    fn exchange_mode(&self,auth:&str,remember:bool,approval:bool,attack:&str,mode:&str)->(Result<Request,String>,usize,Option<String>){
        let code=if auth=="qr"{self.pair.lock().unwrap().qr_code(self.now).unwrap().to_string()}else if auth=="pin"{self.pair.lock().unwrap().pin_code(self.now).unwrap().to_string()}else{String::new()};
        let request=serde_json::json!({"version":6,"code":code,"name":"Test phone","mode":mode,"quality":2,"public_key":self.public,"auth":auth,"remember":remember});
        let signed:Request=serde_json::from_value(request.clone()).unwrap();let (mut server,mut client)=UnixStream::pair().unwrap();server.set_read_timeout(Some(Duration::from_secs(2))).unwrap();client.set_read_timeout(Some(Duration::from_secs(2))).unwrap();let store=self.store.clone();let pairing=self.pair.clone();let pin=self.pin.clone();let now=self.now;let clock=Arc::new(AtomicU64::new(0));let server_clock=clock.clone();let calls=Arc::new(AtomicUsize::new(0));let count=calls.clone();
        let worker=std::thread::spawn(move||pairing::authenticate_with_trust(&mut server,&pairing,Some(&store),&pin,|_|{count.fetch_add(1,Ordering::SeqCst);approval},||now+Duration::from_millis(server_clock.load(Ordering::SeqCst))));
        client.write_all(b"RLY6").unwrap();pairing::write_json(&mut client,&request).unwrap();let mut saved=None;
        if let Ok(challenge)=pairing::read_json::<serde_json::Value>(&mut client){
            let nonce=challenge["nonce"].as_str().unwrap();let mut transcript=pairing::transcript(&self.pin,nonce,&signed);
            if auth=="pin"{assert_eq!(challenge["verifyCode"].as_str().unwrap(),pairing::verification_code(&self.pin,nonce,&self.public));}
            if attack=="direction"{transcript=transcript.replace("\nsend\n","\nview\n");}
            let key=make_key();
            let signature=if let Some(previous)=attack.strip_prefix("replay:"){previous.to_string()}else if attack=="impersonate"{hex(key.sign(&SystemRandom::new(),transcript.as_bytes()).unwrap().as_ref())}else{hex(self.key.sign(&SystemRandom::new(),transcript.as_bytes()).unwrap().as_ref())};saved=Some(signature.clone());
            if attack=="expired-challenge"{clock.store(10000,Ordering::SeqCst);}
            if attack=="revoke-challenge"{self.store.lock().unwrap().forget(&self.id()).unwrap();}
            pairing::write_json(&mut client,&serde_json::json!({"signature":signature})).unwrap();
            if let Ok(response)=pairing::read_json::<serde_json::Value>(&mut client){
                if attack=="expired-session"{clock.store(20000,Ordering::SeqCst);}
                if attack=="revoke-session"{self.store.lock().unwrap().forget(&self.id()).unwrap();}
                pairing::write_json(&mut client,&serde_json::json!({"session":response["session"]})).unwrap();let mut ack=[0;2];let _=client.read_exact(&mut ack);
            }
        }
        drop(client);(worker.join().unwrap(),calls.load(Ordering::SeqCst),saved)
    }
}
fn make_key()->EcdsaKeyPair{let random=SystemRandom::new();let bytes=EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING,&random).unwrap();EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING,bytes.as_ref(),&random).unwrap()}
#[test]fn first_pair_requires_approval_and_persists_only_after_success(){let f=Fixture::new();let(result,count,_)=f.exchange("qr",true,false,"");assert!(result.is_err());assert_eq!(count,1);assert!(f.store.lock().unwrap().peers.is_empty());let(result,count,_)=f.exchange("qr",true,true,"");assert!(result.unwrap().remembered);assert_eq!(count,1);assert!(Store::open(&f.root,&f.pin).unwrap().peers.contains_key(&f.id()));}
#[test]fn pairing_only_requires_pc_approval_and_enrolls_without_a_mirroring_direction(){let f=Fixture::new();let(result,count,_)=f.exchange_mode("qr",true,false,"","pair");assert!(result.is_err());assert_eq!(count,1);assert!(!f.store.lock().unwrap().peers.contains_key(&f.id()));let(result,count,_)=f.exchange_mode("qr",true,true,"","pair");let paired=result.unwrap();assert_eq!(paired.mode,"pair");assert!(paired.remembered);assert_eq!(count,1);assert!(f.store.lock().unwrap().peers.contains_key(&f.id()));}
#[test]fn six_digit_pin_pairing_requires_matching_verification_and_pc_approval(){let f=Fixture::new();let(result,count,_)=f.exchange_mode("pin",true,false,"","pair");assert!(result.is_err());assert_eq!(count,1);assert!(!f.store.lock().unwrap().peers.contains_key(&f.id()));let(result,count,_)=f.exchange_mode("pin",true,true,"","pair");assert_eq!(result.unwrap().mode,"pair");assert_eq!(count,1);assert!(f.store.lock().unwrap().peers.contains_key(&f.id()));}
#[test]fn pairing_only_rejects_unremembered_or_already_trusted_requests(){let f=Fixture::new();assert!(f.exchange_mode("qr",false,true,"","pair").0.is_err());assert!(f.exchange("qr",true,true,"").0.is_ok());assert!(f.exchange_mode("trusted",false,true,"","pair").0.is_err());}
#[test]fn remembered_reconnection_uses_key_without_qr_or_approval(){let f=Fixture::new();assert!(f.exchange("qr",true,true,"").0.is_ok());for _ in 0..2{let(result,count,_)=f.exchange("trusted",false,false,"");assert!(result.unwrap().remembered);assert_eq!(count,0);}}
#[test]fn approval_policy_is_enforced_on_remembered_devices(){let f=Fixture::new();assert!(f.exchange("qr",true,true,"").0.is_ok());f.store.lock().unwrap().set_approval(true).unwrap();let(result,count,_)=f.exchange("trusted",false,false,"");assert!(result.is_err());assert_eq!(count,1);assert!(f.exchange("trusted",false,true,"").0.is_ok());}
#[test]fn impersonation_direction_tampering_replay_and_expiration_are_rejected(){let f=Fixture::new();assert!(f.exchange("qr",true,true,"").0.is_ok());let(_,_,signature)=f.exchange("trusted",false,false,"");for attack in ["impersonate","direction","expired-challenge","expired-session"]{assert!(f.exchange("trusted",false,false,attack).0.is_err(),"Attack was accepted: {attack}");}assert!(f.exchange("trusted",false,false,&format!("replay:{}",signature.unwrap())).0.is_err());}
#[test]fn revocation_requires_repair_and_rechecks_inflight_authorization(){for stage in ["revoke-challenge","revoke-session"]{let f=Fixture::new();assert!(f.exchange("qr",true,true,"").0.is_ok());assert!(f.exchange("trusted",false,false,stage).0.is_err());assert!(!Store::open(&f.root,&f.pin).unwrap().peers.contains_key(&f.id()));assert!(f.exchange("trusted",false,true,"").0.is_err());assert!(f.exchange("qr",true,false,"").0.is_err());assert!(f.exchange("qr",true,true,"").0.is_ok());}}
#[test]fn pc_certificate_change_and_device_key_change_invalidate_trust(){let f=Fixture::new();assert!(f.exchange("qr",true,true,"").0.is_ok());let changed=Store::open(&f.root,&"2".repeat(64)).unwrap();assert!(changed.peers.is_empty());assert!(Store::open(&f.root,&f.pin).unwrap().peers.is_empty());let other=make_key();let other_id=peer_id(&hex(other.public_key().as_ref())).unwrap();assert_ne!(f.id(),other_id);}
#[test]fn private_store_permissions_and_invalid_store_fail_closed(){use std::os::unix::fs::PermissionsExt;let f=Fixture::new();assert!(f.exchange("qr",true,true,"").0.is_ok());assert_eq!(std::fs::metadata(&f.root).unwrap().permissions().mode()&0o777,0o700);assert_eq!(std::fs::metadata(f.root.join("trusted-devices.json")).unwrap().permissions().mode()&0o777,0o600);std::fs::write(f.root.join("trusted-devices.json"),b"invalid").unwrap();assert!(Store::open(&f.root,&f.pin).is_err());}
#[test]fn java_der_signature_vector_verifies_in_rust_and_binds_identity(){let lines:Vec<_>=include_str!("../tests/fixtures/java-p256-proof.txt").lines().collect();let message=String::from_utf8(decode(lines[2]).unwrap()).unwrap();verify(lines[0],&message,lines[1]).unwrap();assert!(verify(lines[0],&message.replace("\nview\n","\nsend\n"),lines[1]).is_err());}

#[test]
fn session_only_pairing_does_not_enroll_and_both_directions_keep_signed_authentication(){
    let f=Fixture::new();
    assert!(!f.exchange("qr",false,true,"").0.unwrap().remembered);
    assert!(f.store.lock().unwrap().peers.is_empty());
    assert!(f.exchange("trusted",false,true,"").0.is_err());
    assert!(f.exchange_mode("qr",true,true,"","view").0.is_ok());
    // Reload persisted records as a new desktop process would.
    *f.store.lock().unwrap()=Store::open(&f.root,&f.pin).unwrap();
    let (result,approvals,_)=f.exchange_mode("trusted",false,false,"","view");
    assert_eq!(result.unwrap().mode,"view");assert_eq!(approvals,0);
}
#[test]
fn replacement_phone_key_cannot_inherit_a_remembered_name(){
    let mut f=Fixture::new();assert!(f.exchange("qr",true,true,"").0.is_ok());
    f.key=make_key();f.public=hex(f.key.public_key().as_ref());
    let (result,approvals,_)=f.exchange("trusted",false,true,"");
    assert!(result.is_err());assert_eq!(approvals,0);
    assert!(f.exchange("qr",true,true,"").0.is_ok());
}
