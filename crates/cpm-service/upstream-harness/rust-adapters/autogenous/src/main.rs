use agl_types::{AdmissionError, Applicability, Authority, Genome, HardInvariant, Mutation, MutationScope};
use std::{env,fs};
fn main(){
 let mode=env::args().nth(1).expect("mode");
 let artifact=fs::read(env::var("CPM_CANDIDATE_ARTIFACT").unwrap()).unwrap();
 let manifest=fs::read_to_string(env::var("CPM_CANDIDATE_MANIFEST").unwrap()).unwrap();
 let digest=env::var("CPM_CANDIDATE_ARTIFACT_SHA256").unwrap();
 let parent=Genome{hash:digest.clone(),identity:manifest.clone(),constitution:env::var("CPM_CANDIDATE_MANIFEST_SHA256").unwrap(),capability_ceiling:Authority::Governed,hard_invariants:vec![HardInvariant{name:"candidate-content-bound".into(),holds:true}],lineage:vec![]};
 let mut mutation=Mutation{id:format!("{}-{}",env::var("CPM_CANDIDATE_PLUGIN").unwrap(),artifact.len()),parent_genome_hash:digest.clone(),scope:MutationScope::ApplicationCode,requested_authority:Authority::Governed,applicability:Applicability{workloads:vec![manifest.clone()],environments:vec![format!("bytes:{}",artifact.len())],jurisdictions:vec![]},preserved_invariants:parent.hard_invariants.clone(),rollback_target:Some(digest.clone()),expires_at:Some(4_000_000_000),signature:None};
 if mode=="failure"{mutation.requested_authority=Authority::Constitutional;assert!(matches!(mutation.admissible(&parent,2_000_000_000),Err(AdmissionError::AuthorityExpansion{..})));println!("candidate={} authority-expansion-rejected bytes={}",digest,artifact.len());}
 else{mutation.admissible(&parent,2_000_000_000).expect("candidate plan admissible");println!("candidate={} plan={} manifest-bytes={} artifact-sum={}",digest,mutation.id,manifest.len(),artifact.iter().map(|v|u64::from(*v)).sum::<u64>());}
}
