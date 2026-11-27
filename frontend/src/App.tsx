import { FormEvent, useEffect, useMemo, useState } from "react";
import {
  Admission, ApiError, AuditEvent, CapabilityMeshClient, Intake,
  RegistryEntry, Trial, UpstreamEvidence,
} from "../../api/client";

type View = "admission" | "registry" | "audit" | "mesh";
type Phase = "idle" | "loading" | "success" | "denied" | "error";

const selectedUpstreams: UpstreamEvidence[] = [
  { project: "ruvnet/autogenous", revision: "3155111", responsibility: "capability protocol derivation", status: "failed" },
  { project: "ruvnet/ruGraph", revision: "d66d976", responsibility: "typed provenance graph", status: "failed" },
  { project: "ruvnet/agenticow", revision: "a646786", responsibility: "authority boundary analysis", status: "failed" },
  { project: "ruvnet/ai-text-watermark", revision: "5871cf5", responsibility: "receipt watermark verification", status: "failed" },
  { project: "ruvnet/LatentMesh", revision: "b04a705", responsibility: "signed observation envelopes", status: "failed" },
  { project: "NVIDIA/SkillSpector", revision: "48e0f94", responsibility: "static capability inspection", status: "failed" },
  { project: "sigstore/cosign", revision: "252f10a", responsibility: "artifact attestation", status: "failed" },
];

const defaultManifest = JSON.stringify({
  name: "image-transform",
  version: "1.0.0",
  capabilities: ["blob:read", "blob:write"],
  scenarios: ["resize", "thumbnail"],
}, null, 2);

function describeError(error: unknown): string {
  if (error instanceof ApiError) return `${error.envelope.code}: ${error.envelope.message}`;
  return error instanceof Error ? error.message : "Unexpected request failure";
}

function Banner({ phase, message }: { phase: Phase; message: string }) {
  if (!message) return null;
  return <div className={`banner banner--${phase}`} role={phase === "error" ? "alert" : "status"} aria-live="polite">
    <span aria-hidden="true" /><strong>{phase}</strong><p>{message}</p>
  </div>;
}

function Empty({ title, children }: { title: string; children: string }) {
  return <div className="empty"><div className="mesh-art" aria-hidden="true"><i /><i /><i /><i /></div><strong>{title}</strong><p>{children}</p></div>;
}

function Evidence({ upstreams }: { upstreams: UpstreamEvidence[] }) {
  const evidence = upstreams.length ? upstreams : selectedUpstreams;
  return <div className="evidence-grid" data-testid="upstream-evidence">
    {evidence.map((item) => <article key={item.project}>
      <header><span className={`dot dot--${item.status}`} /><h3>{item.project}</h3></header>
      <p>{item.responsibility}</p><code>{item.revision}</code><small>{item.status}</small>
    </article>)}
  </div>;
}

function TrialResult({ trial }: { trial: Trial }) {
  return <>
    <div className="metrics">
      <div><span>Useful work</span><strong>{Math.round(trial.useful_work_score * 100)}%</strong></div>
      <div><span>Risk</span><strong>{Math.round(trial.risk_score * 100)}%</strong></div>
      <div><span>Effects</span><strong>{trial.effect_count}</strong></div>
    </div>
    {trial.graph.length ? <div className="graph" aria-label="Capability provenance graph">
      {trial.graph.map((edge) => <div className={`edge edge--${edge.kind}`} key={edge.id}>
        <span>{edge.kind}</span><strong>{edge.from}</strong><b aria-hidden="true">-&gt;</b><strong>{edge.to}</strong><small>{edge.evidence}</small>
      </div>)}
    </div> : <Empty title="No capability edges">The frozen scenario pack produced no authority observations.</Empty>}
  </>;
}

export function App({ client: provided }: { client?: CapabilityMeshClient }) {
  const [token, setToken] = useState("cpm-reviewer-token");
  const client = useMemo(() => provided ?? new CapabilityMeshClient({ token }), [provided]);
  useEffect(() => client.setToken(token), [client, token]);

  const [view, setView] = useState<View>("admission");
  const [ready, setReady] = useState<"checking" | "ready" | "offline">("checking");
  const [phase, setPhase] = useState<Phase>("idle");
  const [message, setMessage] = useState("");
  const [plugin, setPlugin] = useState("image-transform");
  const [version, setVersion] = useState("1.0.0");
  const [manifest, setManifest] = useState(defaultManifest);
  const [intake, setIntake] = useState<Intake | null>(null);
  const [trial, setTrial] = useState<Trial | null>(null);
  const [admission, setAdmission] = useState<Admission | null>(null);
  const [registry, setRegistry] = useState<RegistryEntry[]>([]);
  const [registryLoading, setRegistryLoading] = useState(true);
  const [auditId, setAuditId] = useState("");
  const [events, setEvents] = useState<AuditEvent[]>([]);
  const [auditLoading, setAuditLoading] = useState(false);

  async function loadRegistry() {
    setRegistryLoading(true);
    try { setRegistry((await client.getRegistry()).entries); }
    catch { setRegistry([]); }
    finally { setRegistryLoading(false); }
  }

  useEffect(() => {
    void client.ready().then(() => setReady("ready")).catch(() => setReady("offline"));
    void loadRegistry();
  }, [client]);

  async function runTrial(event: FormEvent) {
    event.preventDefault();
    setPhase("loading"); setMessage("Quarantining bytes and executing the frozen scenario pack...");
    setAdmission(null);
    try {
      JSON.parse(manifest);
      const created = await client.createIntake({
        plugin_name: plugin, version, manifest, scenario_pack: "frozen-admission-v1",
      });
      setIntake(created);
      const result = await client.startTrial(created.id);
      setTrial(result);
      setPhase(result.state === "failed" ? "error" : "success");
      setMessage(result.state === "failed"
        ? "The bounded trial failed safely. No admission effect was attempted."
        : "Trial complete. Review observed and transitive authority before deciding.");
    } catch (error) {
      setPhase("error");
      setMessage(error instanceof SyntaxError ? "Manifest must be valid JSON." : describeError(error));
    }
  }

  async function decide(decision: "admitted" | "denied") {
    if (!intake || !trial) return;
    setPhase("loading");
    setMessage(decision === "denied"
      ? "Recording denial before registry activation..."
      : "Signing and durably committing the admitted revision...");
    try {
      const result = await client.decide(intake.id, {
        decision,
        rationale: decision === "denied"
          ? "Observed authority exceeds the reviewed manifest."
          : "Frozen tasks passed with least authority.",
        trial_id: trial.id,
      }, crypto.randomUUID());
      setAdmission(result); setAuditId(result.correlation_id);
      setPhase(result.decision === "denied" ? "denied" : "success");
      setMessage(result.decision === "denied"
        ? "Denied with no registry activation or current-pointer write."
        : "Admitted. The signed receipt is available for durable replay.");
      await loadRegistry();
    } catch (error) { setPhase("error"); setMessage(describeError(error)); }
  }

  function repair() {
    try {
      const next = JSON.parse(manifest) as { capabilities?: string[] };
      next.capabilities = (next.capabilities ?? []).filter((item) => !["network", "clock"].includes(item));
      setManifest(JSON.stringify(next, null, 2));
      setIntake(null); setTrial(null); setAdmission(null); setPhase("idle");
      setMessage("Least-capability repair applied. A new intake and trial are required.");
    } catch { setPhase("error"); setMessage("Repair requires a valid JSON manifest."); }
  }

  async function searchAudit(event: FormEvent) {
    event.preventDefault();
    if (!auditId.trim()) return;
    setAuditLoading(true);
    try { setEvents((await client.getAudit(auditId.trim())).events); }
    catch (error) { setEvents([]); setPhase("error"); setMessage(describeError(error)); }
    finally { setAuditLoading(false); }
  }

  const nav: Array<[View, string, string]> = [
    ["admission", "Admission", "Intake, trial, decision"],
    ["registry", "Registry", "Durable revisions"],
    ["audit", "Audit replay", "Append-only evidence"],
    ["mesh", "Evidence mesh", "Executed upstreams"],
  ];

  return <div className="shell">
    <a className="skip" href="#content">Skip to main content</a>
    <aside>
      <div className="brand"><b>CP</b><div><strong>Capability</strong><span>Provenance Mesh</span></div></div>
      <nav aria-label="Product sections">{nav.map(([key, label, note]) =>
        <button key={key} className={view === key ? "active" : ""} aria-current={view === key ? "page" : undefined} onClick={() => setView(key)}>
          <strong>{label}</strong><span>{note}</span>
        </button>)}
      </nav>
      <div className="service"><span className={`dot dot--${ready === "ready" ? "verified" : "failed"}`} /><div><strong>{ready === "checking" ? "Checking service" : ready === "ready" ? "Service ready" : "Service offline"}</strong><small>API v1 / local boundary</small></div></div>
    </aside>

    <main id="content">
      <header className="top"><div><p className="eyebrow">Admission control plane</p><h1>{nav.find(([key]) => key === view)?.[1]}</h1></div>
        <label>Reviewer token<input type="password" autoComplete="off" value={token} onChange={(event) => setToken(event.target.value)} /></label>
      </header>
      <Banner phase={phase} message={message} />

      {view === "admission" && <div className="workspace">
        <section className="panel">
          <header className="panel-head"><div><p className="eyebrow">01 / Intake</p><h2>Quarantine a capability</h2></div><span className="pill">No ambient authority</span></header>
          <form onSubmit={runTrial}>
            <div className="pair"><label>Plugin name<input required value={plugin} onChange={(event) => setPlugin(event.target.value)} /></label><label>Version<input required value={version} onChange={(event) => setVersion(event.target.value)} /></label></div>
            <label>Capability manifest<textarea required rows={12} spellCheck={false} value={manifest} onChange={(event) => setManifest(event.target.value)} /></label>
            <label>Frozen scenario pack<select defaultValue="frozen-admission-v1"><option value="frozen-admission-v1">Admission pack v1 - image transform</option></select></label>
            <button className="primary" disabled={phase === "loading"}>{phase === "loading" ? "Running evidence chain..." : "Quarantine and run trial"}</button>
          </form>
        </section>

        <section className="panel">
          <header className="panel-head"><div><p className="eyebrow">02 / Provenance</p><h2>Observed authority</h2></div>{trial && <span className="pill good">{trial.state}</span>}</header>
          {trial ? <><TrialResult trial={trial} /><div className="actions"><button className="secondary danger" onClick={() => void decide("denied")}>Deny and preserve quarantine</button><button className="primary" onClick={() => void decide("admitted")}>Admit signed revision</button></div></>
            : <Empty title="Evidence closes here">Submit a manifest to correlate static inspection, bounded execution, graph closure, authority analysis, and attestation.</Empty>}
        </section>

        {admission && <section className={`panel receipt receipt--${admission.decision}`} data-testid="decision-receipt">
          <div><p className="eyebrow">03 / Decision receipt</p><h2>{admission.decision === "admitted" ? "Revision admitted" : "Authority denied"}</h2><p>{admission.decision === "denied" ? "Denial was audited without a forbidden downstream effect." : "Decision and exact evidence are durable."}</p></div>
          <dl><div><dt>Correlation</dt><dd>{admission.correlation_id}</dd></div><div><dt>Receipt digest</dt><dd>{admission.receipt_digest}</dd></div><div><dt>Current pointer</dt><dd data-testid="current-pointer">{String(admission.current_pointer_written)}</dd></div></dl>
          {admission.decision === "denied" && <button className="secondary" onClick={repair}>Apply least-capability repair</button>}
        </section>}
      </div>}

      {view === "registry" && <section className="panel content">
        <header className="panel-head"><div><p className="eyebrow">Durable state</p><h2>Admitted revision registry</h2></div><button className="secondary" onClick={() => void loadRegistry()}>Refresh</button></header>
        {registryLoading ? <p className="loading" role="status">Loading durable revisions...</p> : registry.length === 0
          ? <Empty title="No active revisions">Only admitted, signed revisions appear here. Denials never create a current pointer.</Empty>
          : <div className="registry">{registry.map((entry) => <article key={entry.admission_id}><span className="pill good">active</span><h3>{entry.plugin_name} <small>{entry.version}</small></h3><p>{entry.receipt_digest}</p><button onClick={() => { setAuditId(entry.correlation_id); setView("audit"); }}>Replay audit</button></article>)}</div>}
      </section>}

      {view === "audit" && <section className="panel content">
        <header className="panel-head"><div><p className="eyebrow">Append-only replay</p><h2>Correlation audit</h2></div></header>
        <form className="audit-search" onSubmit={searchAudit}><label>Correlation ID<input value={auditId} onChange={(event) => setAuditId(event.target.value)} placeholder="corr_..." /></label><button className="primary">Retrieve audit</button></form>
        {auditLoading ? <p className="loading">Replaying immutable events...</p> : events.length === 0
          ? <Empty title="No replay loaded">Enter a decision correlation ID to reconstruct its source-bound event chain.</Empty>
          : <ol className="timeline" data-testid="audit-events">{events.map((event) => <li key={event.sequence}><span>{event.sequence}</span><div><strong>{event.kind}</strong><p>{event.detail}</p><small>{event.occurred_at} / {event.subject}</small></div></li>)}</ol>}
      </section>}

      {view === "mesh" && <section className="panel content">
        <header className="panel-head"><div><p className="eyebrow">Source-bound execution</p><h2>Evidence mesh responsibilities</h2></div><span className="pill">{trial?.upstreams.length ?? 0} observed</span></header>
        <p className="lede">A verified state is rendered only from the trial response. Selection metadata alone is not runtime evidence.</p>
        <Evidence upstreams={trial?.upstreams ?? []} />
      </section>}
    </main>
  </div>;
}
