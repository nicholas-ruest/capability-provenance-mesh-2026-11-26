#!/usr/bin/env node
import{createHash}from"node:crypto";
import{existsSync,readFileSync,readdirSync,writeFileSync}from"node:fs";
import{execFileSync}from"node:child_process";
import{resolve}from"node:path";
import{REQUIRED_SUITES,validateEvidenceMap}from"./validate-evidence-map.mjs";

const[rootArg=".",evidenceArg]=process.argv.slice(2);if(!evidenceArg)throw new Error("usage: finalize-evidence.mjs ROOT EVIDENCE");
const root=resolve(rootArg),evidence=resolve(evidenceArg),sha=execFileSync("git",["-C",root,"rev-parse","HEAD"],{encoding:"utf8"}).trim(),producer=process.env.DREAM_MACHINE_PRODUCER_RUN_ID;
if(!producer||producer.length<8)throw new Error("actual DREAM_MACHINE_PRODUCER_RUN_ID is required");
const digest=p=>createHash("sha256").update(readFileSync(p)).digest("hex"),requiredFile=p=>{const f=resolve(evidence,p);if(!existsSync(f)||!readFileSync(f).length)throw new Error("missing nonempty "+p);return f};
const receiptFiles=readdirSync(evidence).filter(x=>x.endsWith(".receipt.json")).sort();if(receiptFiles.length!==REQUIRED_SUITES.length)throw new Error("expected exactly ten suite receipt files");
const receipts=receiptFiles.map(path=>({path,receipt:JSON.parse(readFileSync(requiredFile(path),"utf8"))}));
for(const suite of REQUIRED_SUITES){const matches=receipts.filter(x=>x.receipt.suite===suite);if(matches.length!==1)throw new Error("need exactly one receipt for "+suite)}
for(const{receipt:r}of receipts){if(r.schema_version!==2||r.source_sha!==sha||r.exit_code!==0||r.signal||r.error||r.integrity_error||!Array.isArray(r.argv)||!r.argv.length)throw new Error("failed or stale receipt: "+r.suite);if(digest(requiredFile(r.log))!==r.log_sha256)throw new Error("receipt log hash mismatch: "+r.suite);if(r.stderr_log&&digest(resolve(evidence,r.stderr_log))!==r.stderr_sha256)throw new Error("receipt stderr hash mismatch: "+r.suite)}
const composition=JSON.parse(readFileSync(requiredFile("composition-report.json"),"utf8"));validateEvidenceMap(root,evidence,composition);
const review=readFileSync(requiredFile("semantic-review.md"),"utf8");if(!review.includes(sha))throw new Error("semantic review is not source-bound");
const selection=JSON.parse(readFileSync(requiredFile("selection-report.json"),"utf8")),probe=selection.probe_source_archive;
if(!probe||typeof probe.path!=="string"||!probe.path.startsWith("probes/archive/")||!probe.path.endsWith(".tar.gz")||probe.path.includes("..")||!/^[a-f0-9]{64}$/.test(probe.sha256??""))throw new Error("invalid source-bound probe archive declaration");
if(digest(requiredFile(probe.path))!==probe.sha256)throw new Error("source-bound probe archive hash mismatch");
const specs=[["desktop_screenshot","desktop.png"],["mobile_screenshot","mobile.png"],["browser_report","playwright-report.json"],["browser_trace","playwright-trace.zip"],["correlation_trace","correlation-trace.ndjson"],["semantic_review","semantic-review.md"],["composition_report","composition-report.json"],["selection_report","selection-report.json"],["integration_execution","integrations-trace.ndjson"],["benchmark_results","benchmark-results.json"],["ablation_results","ablation-results.json"],["probe_archive",probe.path]];
const artifacts=specs.map(([kind,path])=>({kind,path,sha256:digest(requiredFile(path))}));
for(const{path}of receipts)artifacts.push({kind:"execution_receipt",path,sha256:digest(requiredFile(path))});
const backend=receipts.find(x=>x.receipt.suite==="backend").receipt;artifacts.push({kind:"backend_metadata",path:backend.log,sha256:digest(requiredFile(backend.log))});
const manifest={schema_version:2,source_sha:sha,producer_run_id:producer,generated_at:new Date().toISOString(),artifacts};
writeFileSync(resolve(evidence,"manifest.json"),JSON.stringify(manifest,null,2)+"\n");
console.log(JSON.stringify({status:"EVIDENCE_FINALIZED",source_sha:sha,producer_run_id:producer,artifacts:artifacts.length}));
