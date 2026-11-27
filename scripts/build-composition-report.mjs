#!/usr/bin/env node
import {readFileSync, writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {resolve} from 'node:path';
const [rootArg='.', evidenceArg, inputArg]=process.argv.slice(2);
if(!evidenceArg||!inputArg) throw new Error('usage: build-composition-report.mjs ROOT EVIDENCE EXECUTION_MAP');
const root=resolve(rootArg), evidence=resolve(evidenceArg);
const sha=execFileSync('git',['-C',root,'rev-parse','HEAD'],{encoding:'utf8'}).trim();
const input=JSON.parse(readFileSync(resolve(inputArg),'utf8'));
if(input.source_sha!==sha||input.schema_version!==2) throw new Error('execution map must be schema 2 and exact-source bound');
if(!Array.isArray(input.dependencies)||input.dependencies.length<7) throw new Error('need real executed dependencies');
if(!Array.isArray(input.meaningful_crates)||input.meaningful_crates.length<20) throw new Error('need >=20 reviewed crates');
for(const d of input.dependencies) for(const k of ['repo','category','revision','license','responsibility','manifest_path','call_site_path','test_path','runtime_evidence_path']) if(!d[k]) throw new Error('missing dependency '+k);
writeFileSync(resolve(evidence,'composition-report.json'),JSON.stringify({...input,schema_version:2,source_sha:sha},null,2)+'\n');
