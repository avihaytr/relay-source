// SPDX-License-Identifier: MPL-2.0
//! Public trusted-peer keys protected by the owning user's private data directory.
use std::{collections::BTreeMap, fs, io::Write, os::unix::fs::{OpenOptionsExt, PermissionsExt, MetadataExt}, path::{Path,PathBuf}};
use serde::{Deserialize,Serialize};
use ring::{digest,signature};
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Peer {pub name:String,pub public_key:String}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Store {pub server_pin:String,pub require_approval:bool,pub generation:u64,pub peers:BTreeMap<String,Peer>,#[serde(skip)]path:PathBuf}
pub fn hex(bytes:&[u8])->String{bytes.iter().map(|b|format!("{b:02x}")).collect()}
pub fn decode(value:&str)->Result<Vec<u8>,String>{if value.len()>1024||value.len()%2!=0{return Err("Invalid identity encoding".into());}value.as_bytes().chunks_exact(2).map(|pair|{let s=std::str::from_utf8(pair).map_err(|_|"Invalid identity encoding")?;u8::from_str_radix(s,16).map_err(|_|"Invalid identity encoding".into())}).collect()}
pub fn peer_id(public:&str)->Result<String,String>{let bytes=decode(public)?;if bytes.len()!=65||bytes[0]!=4{return Err("Invalid P-256 public key".into());}Ok(hex(digest::digest(&digest::SHA256,&bytes).as_ref()))}
pub fn verify(public:&str,message:&str,sig:&str)->Result<(),String>{let key=decode(public)?;let signature=decode(sig)?;signature::UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_ASN1,key).verify(message.as_bytes(),&signature).map_err(|_|"Device identity could not be verified".into())}
pub fn private_directory(root:&Path)->Result<(),String>{fs::create_dir_all(root).map_err(|_|"Cannot create private Relay directory")?;let metadata=fs::symlink_metadata(root).map_err(|_|"Cannot inspect Relay directory")?;if !metadata.is_dir()||metadata.uid()!=unsafe{libc::geteuid()}{return Err("Relay directory must belong to this user and must not be a symlink".into());}fs::set_permissions(root,fs::Permissions::from_mode(0o700)).map_err(|_|"Cannot protect Relay directory".into())}
pub fn protect_file(path:&Path)->Result<(),String>{let metadata=fs::symlink_metadata(path).map_err(|_|"Cannot inspect identity file")?;if !metadata.is_file()||metadata.uid()!=unsafe{libc::geteuid()}{return Err("Identity file must be a regular file owned by this user".into());}fs::set_permissions(path,fs::Permissions::from_mode(0o600)).map_err(|_|"Cannot protect identity file".into())}
impl Store {
    pub fn open(root:&Path,pin:&str)->Result<Self,String>{
        private_directory(root)?;let path=root.join("trusted-devices.json");
        let mut store=if path.exists(){protect_file(&path)?;let bytes=fs::read(&path).map_err(|_|"Cannot read trusted devices")?;if bytes.len()>128*1024{return Err("Invalid trusted-device store".into());}serde_json::from_slice::<Self>(&bytes).map_err(|_|"Invalid trusted-device store; reset it before pairing")?}else{Self{server_pin:pin.into(),require_approval:false,generation:0,peers:BTreeMap::new(),path:path.clone()}};
        if store.peers.len()>128||store.peers.iter().any(|(id,p)|peer_id(&p.public_key).ok().as_ref()!=Some(id)||p.name.is_empty()||p.name.len()>50||p.name.chars().any(|c|c.is_control())){return Err("Invalid trusted-device identity records".into());}
        store.path=path;
        // New PC certificate/private key means a new identity, never inherited trust.
        if store.server_pin!=pin{store.server_pin=pin.into();store.peers.clear();store.generation=store.generation.wrapping_add(1);store.save()?;}
        Ok(store)
    }
    fn save(&self)->Result<(),String>{
        let bytes=serde_json::to_vec(self).map_err(|_|"Cannot encode trusted devices")?;
        let nonce=crate::pairing::random_hex()?;let temp=self.path.with_extension(format!("{nonce}.tmp"));
        let result=(||{let mut file=fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temp).map_err(|_|"Cannot create trusted-device file")?;file.write_all(&bytes).and_then(|_|file.sync_all()).map_err(|_|"Cannot save trusted devices")?;fs::rename(&temp,&self.path).map_err(|_|"Cannot replace trusted-device file")?;Ok(())})();if result.is_err(){let _=fs::remove_file(temp);}result
    }
    pub fn remember(&mut self,name:&str,public:&str)->Result<(),String>{if name.is_empty()||name.len()>50||name.chars().any(|c|c.is_control()){return Err("Invalid device name".into());}let id=peer_id(public)?;if self.peers.len()>=128&&!self.peers.contains_key(&id){return Err("Trusted-device limit reached".into());}let mut next=self.clone();next.peers.insert(id,Peer{name:name.into(),public_key:public.into()});next.generation=next.generation.wrapping_add(1);next.save()?;*self=next;Ok(())}
    pub fn forget(&mut self,id:&str)->Result<(),String>{let mut next=self.clone();next.peers.remove(id);next.generation=next.generation.wrapping_add(1);next.save()?;*self=next;Ok(())}
    pub fn set_approval(&mut self,required:bool)->Result<(),String>{let mut next=self.clone();next.require_approval=required;next.generation=next.generation.wrapping_add(1);next.save()?;*self=next;Ok(())}
}

#[cfg(test)]
#[path="trust_tests.rs"]
mod tests;
