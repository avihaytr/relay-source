// SPDX-License-Identifier: MPL-2.0
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Profile {pub id:u32,pub label:&'static str,pub width:u32,pub height:u32,pub fps:u32,pub kbps:u32}
pub fn get(id:u32)->Profile {match id {
    0=>Profile{id:0,label:"Low latency · 30 FPS",width:1280,height:720,fps:30,kbps:4000},
    1=>Profile{id:1,label:"HD · 60 FPS",width:1280,height:720,fps:60,kbps:8000},
    _=>Profile{id:2,label:"Full HD · 60 FPS",width:1920,height:1080,fps:60,kbps:16000},
}}
pub fn device_name(value:&str)->String {
    let mut name=String::new();
    for character in value.trim().chars().filter(|c|!c.is_control()) {
        if name.len()+character.len_utf8()>50{break;}name.push(character);
    }
    if name.trim().is_empty(){random_name()}else{name.trim().into()}
}
pub fn random_name()->String{
    const ADJECTIVES:&[&str]=&["Amber","Brisk","Calm","Clever","Cloudy","Cozy","Cosmic","Daring","Dazzling","Gentle","Golden","Grumpy","Happy","Jolly","Lucky","Merry","Misty","Nimble","Quiet","Quick","Silver","Sunny","Velvet","Witty","Zealous","Bouncy","Curious","Dreamy","Funky","Fuzzy","Kind","Lively","Sleepy","Tiny","Wise"];
    const NOUNS:&[&str]=&["Otter","Badger","Heron","Panda","Robin","Marten","Falcon","Koala","Lynx","Puffin","Dolphin","Gecko","Marmot","Raven","Turtle","Fox","Wombat","Finch","Orca","Lemur","Moose","Owl","Seal","Yak","Ferret","Hedgehog","Kingfisher","Osprey","Quokka","Sparrow","Walrus","Weasel","Penguin"];
    let mut bytes=[0u8;8];if std::fs::File::open("/dev/urandom").and_then(|mut f|std::io::Read::read_exact(&mut f,&mut bytes)).is_err(){bytes=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos().to_le_bytes()[..8].try_into().unwrap_or([0;8]);}
    let left=u64::from_le_bytes(bytes) as usize%ADJECTIVES.len();let right=u32::from_le_bytes(bytes[4..].try_into().unwrap()) as usize%NOUNS.len();format!("{} {}",ADJECTIVES[left],NOUNS[right])
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn hd_profiles_request_real_60fps_and_more_bandwidth(){assert_eq!(get(2).width,1920);assert_eq!(get(2).height,1080);assert_eq!(get(2).fps,60);assert!(get(2).kbps>get(1).kbps);assert_eq!(get(0).fps,30);}
    #[test] fn device_names_are_safe_for_mdns_without_cutting_unicode(){let name=device_name(&"אב".repeat(40));assert!(name.len()<=50);assert!(!device_name("  My PC\n").contains('\n'));let random=device_name("\n  ");assert!(random.split_whitespace().count()==2);assert!(random.chars().next().unwrap().is_uppercase());}
}
