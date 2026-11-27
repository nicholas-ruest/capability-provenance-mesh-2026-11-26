use ruflo_watermark::{detect_gumbel,Scheme,WatermarkConfig,WatermarkKey,Watermarker};
use std::{env,fs};
fn main(){
 let mode=env::args().nth(1).expect("mode");let artifact=fs::read(env::var("CPM_CANDIDATE_ARTIFACT").unwrap()).unwrap();let manifest=fs::read(env::var("CPM_CANDIDATE_MANIFEST").unwrap()).unwrap();
 let key=WatermarkKey::from_bytes(&manifest);let cfg=WatermarkConfig::new(key.clone());let mut wm=Watermarker::new(cfg.clone(),Scheme::Gumbel);let mut emitted=Vec::new();
 for i in 0..256{let b=u32::from(artifact[i%artifact.len()]);let candidates=[b+1,b+257,b+513,b+769];let idx=wm.step(&candidates,&[0.25;4]);emitted.push(candidates[idx]);}
 if mode=="failure"{let wrong=WatermarkConfig::new(WatermarkKey::from_bytes(b"candidate-wrong-key"));let rejected=detect_gumbel(&emitted,wrong);assert!(!rejected.is_watermarked(1e-6));println!("candidate={} wrong-key-rejected z={}",env::var("CPM_CANDIDATE_ARTIFACT_SHA256").unwrap(),rejected.z_score);}
 else{let detected=detect_gumbel(&emitted,cfg);assert!(detected.scored_positions>100);println!("candidate={} positions={} z={} tokens={:?}",env::var("CPM_CANDIDATE_ARTIFACT_SHA256").unwrap(),detected.scored_positions,detected.z_score,&emitted[..8]);}
}
