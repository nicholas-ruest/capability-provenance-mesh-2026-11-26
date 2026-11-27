#!/usr/bin/env node
import {createHash} from "node:crypto";
import {readFileSync} from "node:fs";
const files=["api/openapi.yaml","api/client/schema.ts","api/client/runtime.ts","crates/cpm-service/src/lib.rs"];
const texts=files.map((p)=>readFileSync(p,"utf8")); const [openapi,schema,client,service]=texts;
const paths=["/readyz","/api/v1/intakes","/api/v1/registry/{plugin}/{version}","/api/v1/audit/{correlation}"];
for(const p of paths){if(!openapi.includes(p))throw new Error("OpenAPI missing "+p);if(!service.includes(p))throw new Error("service route drift "+p);}
for(const name of ["CapabilitySubmission","DecisionRecord","ErrorEnvelope"])if(!schema.includes(name))throw new Error("schema type missing "+name);
for(const method of ["ready(","submit(","getDecision(","getAudit("])if(!client.includes(method))throw new Error("client method missing "+method);
const digest=(s)=>createHash("sha256").update(s).digest("hex");
console.log(JSON.stringify({status:"API_CONTRACT_CHECK_PASS",files:Object.fromEntries(files.map((p,i)=>[p,digest(texts[i])]))},null,2));
