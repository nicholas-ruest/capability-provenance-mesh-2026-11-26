import fs from 'node:fs';
import path from 'node:path';
const upstream=process.argv[2],mode=process.argv[3],{open}=await import(path.join(upstream,'src/index.js'));
const artifact=fs.readFileSync(process.env.CPM_CANDIDATE_ARTIFACT),dim=16;
const vector=Array.from({length:dim},(_,i)=>((artifact[i%artifact.length]??i)+1)/256);
const dir=fs.mkdtempSync(path.join(process.env.CPM_CANDIDATE_DIR,'cow-'));
const base=open(path.join(dir,'base.rvf'),{dimension:dim});base.ingest([{id:1,vector:Array(dim).fill(0.001),text:'baseline'}]);
const before=base.status().totalVectors,branch=base.fork(`candidate/${process.env.CPM_CANDIDATE_ARTIFACT_SHA256.slice(0,12)}`);
branch.ingest([{id:2,vector,text:process.env.CPM_CANDIDATE_MANIFEST_SHA256}]);
if(mode==='failure'){
 const branchPath=branch.lineage()[0].path;branch.close();fs.rmSync(branchPath,{force:true});
 if(base.status().totalVectors!==before)throw new Error('rejected candidate changed durable base');
 console.log(JSON.stringify({candidate:process.env.CPM_CANDIDATE_ARTIFACT_SHA256,rejected:true,durable_vectors:before}));
}else{
 const promoted=branch.promote(base),hit=base.query(vector,1)[0];if(hit.id!==2)throw new Error('candidate vector was not promoted');
 console.log(JSON.stringify({candidate:process.env.CPM_CANDIDATE_ARTIFACT_SHA256,promoted:promoted.ingested,hit:hit.id,lineage:branch.lineage()[0].id}));branch.close();
}
base.close();
