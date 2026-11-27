#!/usr/bin/env node
import{readFileSync,writeFileSync}from"node:fs";
import{resolve}from"node:path";
import{execFileSync}from"node:child_process";
import{validateEvidenceMap}from"./validate-evidence-map.mjs";

const[rootArg=".",evidenceArg,inputArg]=process.argv.slice(2);
if(!evidenceArg||!inputArg)throw new Error("usage: build-composition-report.mjs ROOT EVIDENCE EXECUTION_MAP");
const root=resolve(rootArg),evidence=resolve(evidenceArg),sha=execFileSync("git",["-C",root,"rev-parse","HEAD"],{encoding:"utf8"}).trim();
const report=JSON.parse(readFileSync(resolve(inputArg),"utf8"));
if(report.schema_version!==2||report.source_sha!==sha)throw new Error("stale execution map");
const result=validateEvidenceMap(root,evidence,report);
const f=result.frontier;if(!f||f.benchmark_decisions!==40||f.frontier_decisions!==25||f.component_ablations!==4||f.upstream_ablations!==7||f.balanced_quality_gain_bps!==5000||f.component_quality_delta_bps!==-1250)throw new Error("frontier evidence invariants not proven");
writeFileSync(resolve(evidence,"composition-report.json"),JSON.stringify(report,null,2)+"\n");
console.log(JSON.stringify({status:"COMPOSITION_RECORDED",source_sha:sha,...result}));
