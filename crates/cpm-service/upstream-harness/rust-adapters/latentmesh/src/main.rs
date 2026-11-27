use latentmesh_meshtastic::{decode_single_frame,write_frame};
use std::{env,fs};
fn main(){
 let mode=env::args().nth(1).expect("mode");let mut payload=fs::read(env::var("CPM_CANDIDATE_ARTIFACT").unwrap()).unwrap();payload.extend(fs::read(env::var("CPM_CANDIDATE_MANIFEST").unwrap()).unwrap());
 let mut framed=Vec::new();write_frame(&mut framed,&payload).expect("frame candidate");
 if mode=="failure"{framed[0]^=1;assert!(decode_single_frame(&framed).is_err());println!("candidate={} corrupt-frame-rejected bytes={}",env::var("CPM_CANDIDATE_ARTIFACT_SHA256").unwrap(),payload.len());}
 else{let decoded=decode_single_frame(&framed).expect("decode candidate frame");assert_eq!(decoded,payload);println!("candidate={} frame-bytes={} payload-checksum={}",env::var("CPM_CANDIDATE_ARTIFACT_SHA256").unwrap(),framed.len(),payload.iter().map(|v|u64::from(*v)).sum::<u64>());}
}
