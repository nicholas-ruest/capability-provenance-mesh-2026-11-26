#!/usr/bin/env node
import {createHash} from "node:crypto";
import {readFileSync,writeFileSync,existsSync,readdirSync} from "node:fs";
import {execFileSync} from "node:child_process";
import {resolve} from "node:path";
const [rootArg=".",evidenceArg]=process.argv.slice(2); if(!evidenceArg)throw new Error("usage: finalize-evidence.mjs ROOT EVIDENCE");
const root=resolve(rootArg),evidence=resolve(evidenceArg),sha=execFileSync("git",["-C",root,"rev-parse","HEAD"],{encoding:"utf8"}).trim();
const digest=(p)=>createHash("sha256").update(readFileSync(p)).digest("hex");
const specs=[["desktop_screenshot","desktop.png"],["mobile_screenshot","mobile.png"],["browser_report","playwright-report.json"],["browser_trace","playwright-trace.zip"],["correlation_trace","correlation-trace.ndjson"],["semantic_review","semantic-review.md"],["composition_report","composition-report.json"],["selection_report","selection-report.json"],["integration_execution","integrations-trace.ndjson"]];
const artifacts=specs.map(([kind,path])=>{const p=resolve(evidence,path);if(!existsSync(p)||!readFileSync(p).length)throw new Error("missing nonempty "+path);return{kind,path,sha256:digest(p)}});
const receipts=readdirSync(evidence).filter((x)=>x.endsWith(".receipt.json"));
for(const path of receipts)artifacts.push({kind:"execution_receipt",path,sha256:digest(resolve(evidence,path))});
const backend=receipts.map((p)=>JSON.parse(readFileSync(resolve(evidence,p),"utf8"))).find((r)=>r.suite==="backend"&&r.source_sha===sha);
if(!backend)throw new Error("missing source-bound backend receipt");
artifacts.push({kind:"backend_metadata",path:backend.log,sha256:backend.log_sha256});
writeFileSync(resolve(evidence,"manifest.json"),JSON.stringify({schema_version:2,source_sha:sha,producer_run_id:process.env.DREAM_MACHINE_PRODUCER_RUN_ID??null,generated_at:new Date().toISOString(),artifacts},null,2)+"\n");
