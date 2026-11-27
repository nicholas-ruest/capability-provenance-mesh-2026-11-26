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
writeFileSync(resolve(evidence,"composition-report.json"),JSON.stringify(report,null,2)+"\n");
console.log(JSON.stringify({status:"COMPOSITION_RECORDED",source_sha:sha,...result}));
