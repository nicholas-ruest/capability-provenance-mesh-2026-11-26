import { readFile } from 'node:fs/promises';
import path from 'node:path';
const upstream=process.argv[2], mode=process.argv[3];
const { RuGraph } = await import(path.join(upstream,'npm/packages/graph-language/index.mjs'));
const manifest=JSON.parse(await readFile(process.env.CPM_CANDIDATE_MANIFEST,'utf8'));
const artifact=await readFile(process.env.CPM_CANDIDATE_ARTIFACT);
const plugin=process.env.CPM_CANDIDATE_PLUGIN, caps=manifest.capabilities??['blob.read'];
const ids=caps.map((_,i)=>`cap-${i}`);
const source=`program provenance { effect graph.read; effect decision.emit; budget 100000; let plugin = select ["${plugin}"]; let capabilities = traverse plugin "DECLARES" depth 1; let observed = observe capabilities "Observed" within 300; let decisions = assess observed; emit decisions; }`;
const language=new RuGraph({binary:process.env.RUVECTOR_LANGUAGE_BIN,timeoutMs:30000});
const program=await language.compile(source);
if(mode==='failure'){
 try{await language.run({program,snapshot:{version:1,revision:process.env.CPM_CANDIDATE_ARTIFACT_SHA256,nodes:[],edges:[],observations:[],rules:[]},context:{now:1000,capabilities:[],max_steps:100000,max_results:1000}});throw new Error('unexpected execution');}
 catch(error){if(String(error).includes('unexpected execution'))throw error;console.log(JSON.stringify({candidate:process.env.CPM_CANDIDATE_ARTIFACT_SHA256,rejected:true,code:error.code??'policy'}));}
}else{
 const nodes=[{id:plugin,label:'Plugin',vector:[artifact.length,caps.length]},...ids.map((id,i)=>({id,label:caps[i],vector:[i+1,artifact[i%artifact.length]??0]}))];
 const edges=ids.map((id,i)=>({id:`edge-${i}`,source:plugin,target:id,relation:'DECLARES'}));
 const observations=ids.map((id,i)=>({id:`obs-${i}`,subject:id,predicate:'Observed',value:true,source:process.env.CPM_CANDIDATE_MANIFEST_SHA256,observed_at:990,expires_at:1100}));
 const request={program,snapshot:{version:1,revision:process.env.CPM_CANDIDATE_ARTIFACT_SHA256,nodes,edges,observations,rules:[]},context:{now:1000,capabilities:['graph.read','decision.emit'],max_steps:100000,max_results:1000}};
 const receipt=await language.run(request);await language.replay(request,receipt);
 console.log(JSON.stringify({candidate:process.env.CPM_CANDIDATE_ARTIFACT_SHA256,receipt:receipt.receipt_hash,traversed:edges.length}));
}
