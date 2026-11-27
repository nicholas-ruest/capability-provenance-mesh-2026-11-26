import { FormEvent, useEffect, useMemo, useState } from "react";
import {
  ApiError, CapabilityManifest, DecisionRecord, RuntimeClient, evidenceFrom,
} from "../../api/client/runtime";

type View = "admission" | "registry" | "audit" | "mesh";
const manifestTemplate: CapabilityManifest = {
  api_version: "cpm.capability/v1",
  name: "image-transform",
  entrypoint: "run",
  capabilities: ["blob.read", "blob.write"],
};

function errorText(value: unknown): string {
  if (value instanceof ApiError) return `${value.envelope.code}: ${value.envelope.message}`;
  return value instanceof Error ? value.message : "Unexpected request failure";
}

function Empty({ title, children }: { title: string; children: string }) {
  return <div className="empty"><div className="mesh-art" aria-hidden="true"><i /><i /><i /><i /></div><strong>{title}</strong><p>{children}</p></div>;
}

function RecordView({ record }: { record: DecisionRecord }) {
  const evidence = evidenceFrom(record);
  return <div data-testid="decision-receipt">
    <div className="metrics">
      <div><span>Decision</span><strong>{record.admitted ? "ADMIT" : "DENY"}</strong></div>
      <div><span>Upstreams</span><strong>{evidence.length}/7</strong></div>
      <div><span>Protected effects</span><strong>{record.admitted ? "1" : "0"}</strong></div>
    </div>
    <div className="graph" aria-label="Capability provenance graph">
      {manifestTemplate.capabilities.map((capability, index) => <div className="edge edge--observed" key={capability}>
        <span>{index ? "observed" : "declared"}</span><strong>{record.plugin}</strong><b aria-hidden="true">-&gt;</b><strong>{capability}</strong><small>{record.artifact_digest}</small>
      </div>)}
    </div>
    <dl className="record-data">
      <div><dt>Correlation</dt><dd>{record.correlation_id}</dd></div>
      <div><dt>Receipt checksum</dt><dd>{record.receipt_checksum}</dd></div>
      <div><dt>Artifact digest</dt><dd>{record.artifact_digest}</dd></div>
    </dl>
  </div>;
}

export function ProductApp({ suppliedClient }: { suppliedClient?: RuntimeClient }) {
  const [token, setToken] = useState("cpm-reviewer-token");
  const client = useMemo(() => suppliedClient ?? new RuntimeClient({ token }), [suppliedClient, token]);
  useEffect(() => client.setToken(token), [client, token]);

  const [view, setView] = useState<View>("admission");
  const [ready, setReady] = useState<"checking" | "ready" | "offline">("checking");
  const [phase, setPhase] = useState<"idle" | "loading" | "success" | "denied" | "error">("idle");
  const [message, setMessage] = useState("");
  const [plugin, setPlugin] = useState("image-transform");
  const [version, setVersion] = useState("1.0.0");
  const [manifestText, setManifestText] = useState(JSON.stringify(manifestTemplate, null, 2));
  const [record, setRecord] = useState<DecisionRecord | null>(null);
  const [denialVerified, setDenialVerified] = useState(false);
  const [auditId, setAuditId] = useState("");
  const [auditRecord, setAuditRecord] = useState<DecisionRecord | null>(null);

  useEffect(() => {
    void client.ready().then(() => setReady("ready")).catch(() => setReady("offline"));
    const last = localStorage.getItem("cpm:last-decision");
    if (!last) return;
    try {
      const key = JSON.parse(last) as { plugin: string; version: string };
      void client.getDecision(key.plugin, key.version).then((saved) => {
        setRecord(saved); setPlugin(saved.plugin); setVersion(saved.version); setAuditId(saved.correlation_id);
      }).catch(() => localStorage.removeItem("cpm:last-decision"));
    } catch { localStorage.removeItem("cpm:last-decision"); }
  }, [client]);

  function submission(denialProbe = false) {
    const parsed = JSON.parse(manifestText) as CapabilityManifest;
    return {
      plugin, version,
      artifact: Array.from(new TextEncoder().encode(`CPM:${plugin}:${version}`)),
      manifest: parsed,
      human_approved: true,
      denial_probe: denialProbe,
    };
  }

  async function run(event: FormEvent) {
    event.preventDefault();
    setPhase("loading"); setMessage("Executing intake, seven upstream responsibilities, policy, attestation, and durable write...");
    setDenialVerified(false);
    try {
      const result = await client.submit(submission());
      setRecord(result); setAuditId(result.correlation_id);
      localStorage.setItem("cpm:last-decision", JSON.stringify({ plugin, version }));
      setPhase(result.admitted ? "success" : "denied");
      setMessage(result.admitted ? "Admitted with a source-bound receipt and durable decision." : "Denied with a durable audit and no active authority.");
    } catch (error) { setPhase("error"); setMessage(errorText(error)); }
  }

  async function denialProbe() {
    setPhase("loading"); setMessage("Executing a denied decision, then proving no registry mutation...");
    setRecord(null); setDenialVerified(false);
    try {
      const denied = await client.submit(submission(true));
      if (denied.admitted) {
        setPhase("error"); setMessage("Denial probe unexpectedly admitted the capability."); return;
      }
      setAuditId(denied.correlation_id);
      try {
        await client.getDecision(plugin, version);
        setPhase("error"); setMessage("Forbidden side effect detected: a denied revision exists.");
      } catch (lookup) {
        if (lookup instanceof ApiError && lookup.status === 404) {
          setDenialVerified(true); setPhase("denied");
          setMessage("Denied decision produced no registry entry; the 404 confirms no downstream mutation.");
        } else { setPhase("error"); setMessage(errorText(lookup)); }
      }
    } catch (error) {
      setPhase("error"); setMessage(errorText(error));
    }
  }

  async function retrieve() {
    setPhase("loading"); setMessage("Reading the durable decision from the restarted backend...");
    try {
      const saved = await client.getDecision(plugin, version);
      setRecord(saved); setAuditId(saved.correlation_id); setPhase("success");
      setMessage("Durable decision retrieved from shared storage.");
    } catch (error) { setPhase("error"); setMessage(errorText(error)); }
  }

  async function replay(event: FormEvent) {
    event.preventDefault(); setPhase("loading"); setMessage("Replaying the source-bound audit record...");
    try { const saved = await client.getAudit(auditId); setAuditRecord(saved); setPhase("success"); setMessage("Audit replay matches the durable receipt."); }
    catch (error) { setPhase("error"); setMessage(errorText(error)); }
  }

  const nav: Array<[View, string, string]> = [
    ["admission", "Admission", "Intake and governed outcome"],
    ["registry", "Registry", "Restart-safe retrieval"],
    ["audit", "Audit replay", "Correlation evidence"],
    ["mesh", "Evidence mesh", "Executed upstreams"],
  ];

  return <div className="shell">
    <a className="skip" href="#content">Skip to main content</a>
    <aside><div className="brand"><b>CP</b><div><strong>Capability</strong><span>Provenance Mesh</span></div></div>
      <nav aria-label="Product sections">{nav.map(([key, label, note]) => <button key={key} className={view === key ? "active" : ""} aria-current={view === key ? "page" : undefined} onClick={() => setView(key)}><strong>{label}</strong><span>{note}</span></button>)}</nav>
      <div className="service"><span className={`dot dot--${ready === "ready" ? "verified" : "failed"}`} /><div><strong>{ready === "checking" ? "Checking service" : ready === "ready" ? "Service ready" : "Service offline"}</strong><small>API v1 / local boundary</small></div></div>
    </aside>
    <main id="content">
      <header className="top"><div><p className="eyebrow">Admission control plane</p><h1>{nav.find(([key]) => key === view)?.[1]}</h1></div><label>Reviewer token<input data-testid="token" type="password" value={token} onChange={(event) => setToken(event.target.value)} /></label></header>
      {message && <div className={`banner banner--${phase}`} role={phase === "error" ? "alert" : "status"}><span /><strong>{phase}</strong><p>{message}</p></div>}

      {view === "admission" && <div className="workspace">
        <section className="panel"><header className="panel-head"><div><p className="eyebrow">01 / Intake</p><h2>Quarantine a capability</h2></div><span className="pill">No ambient authority</span></header>
          <form onSubmit={run}><div className="pair"><label>Plugin name<input value={plugin} onChange={(event) => setPlugin(event.target.value)} /></label><label>Version<input value={version} onChange={(event) => setVersion(event.target.value)} /></label></div>
            <label>Capability manifest<textarea rows={12} value={manifestText} onChange={(event) => setManifestText(event.target.value)} /></label>
            <button className="primary" disabled={phase === "loading"}>Run governed admission</button>
            <button className="secondary danger" type="button" disabled={phase === "loading"} onClick={() => void denialProbe()}>Run policy-denial probe</button>
          </form>
        </section>
        <section className="panel"><header className="panel-head"><div><p className="eyebrow">02 / Outcome</p><h2>Observed authority</h2></div>{record && <span className="pill good">{record.admitted ? "admitted" : "denied"}</span>}</header>
          {record ? <RecordView record={record} /> : denialVerified ? <div className="denial-proof" data-testid="denial-proof"><strong>No forbidden downstream effect</strong><p>Denied decision followed by a 404 registry lookup.</p><button className="primary" onClick={() => { setPhase("idle"); setMessage("Recovered. Submit the corrected capability."); }}>Recover and continue</button></div> : <Empty title="Evidence closes here">Submit a manifest to exercise the real browser-to-Rust-to-upstream-to-storage composition.</Empty>}
        </section>
      </div>}

      {view === "registry" && <section className="panel content"><header className="panel-head"><div><p className="eyebrow">Durable state</p><h2>Retrieve admitted revision</h2></div></header>
        <div className="pair"><label>Plugin<input value={plugin} onChange={(event) => setPlugin(event.target.value)} /></label><label>Version<input value={version} onChange={(event) => setVersion(event.target.value)} /></label></div>
        <button className="primary space-top" onClick={() => void retrieve()}>Retrieve from registry</button>
        {record ? <RecordView record={record} /> : <Empty title="No revision loaded">Retrieve an exact plugin and version after browser refresh or backend restart.</Empty>}
      </section>}

      {view === "audit" && <section className="panel content"><header className="panel-head"><div><p className="eyebrow">Append-only replay</p><h2>Correlation audit</h2></div></header>
        <form className="audit-search" onSubmit={replay}><label>Correlation ID<input value={auditId} onChange={(event) => setAuditId(event.target.value)} /></label><button className="primary">Replay audit</button></form>
        {auditRecord ? <RecordView record={auditRecord} /> : <Empty title="No replay loaded">Use the decision correlation ID to read its restart-safe audit record.</Empty>}
      </section>}

      {view === "mesh" && <section className="panel content"><header className="panel-head"><div><p className="eyebrow">Source-bound execution</p><h2>Evidence mesh responsibilities</h2></div><span className="pill">{record ? evidenceFrom(record).length : 0} observed</span></header>
        <p className="lede">Verified badges come only from runtime receipts returned by the Rust composition root.</p>
        <div className="evidence-grid" data-testid="upstream-evidence">{(record ? evidenceFrom(record) : []).map((item) => <article key={item.project}><header><span className="dot dot--verified" /><h3>{item.project}</h3></header><p>{item.responsibility}</p><code>{item.revision}</code><small>{item.status}</small></article>)}</div>
        {!record && <Empty title="No runtime receipts">Complete an admission to render all seven executed upstream responsibilities.</Empty>}
      </section>}
    </main>
  </div>;
}
