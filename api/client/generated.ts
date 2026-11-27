// Generated from api/openapi-v1.json. Do not hand-edit.
export type Decision = "admitted" | "denied";
export type TrialState = "queued" | "running" | "completed" | "failed" | "cancelled";
export type EdgeKind = "declared" | "observed" | "inferred" | "forbidden";

export interface CapabilityEdge {
  id: string;
  from: string;
  to: string;
  kind: EdgeKind;
  evidence: string;
}

export interface UpstreamEvidence {
  project: string;
  revision: string;
  responsibility: string;
  status: "verified" | "failed";
}

export interface IntakeRequest {
  plugin_name: string;
  version: string;
  manifest: string;
  scenario_pack: string;
}

export interface Intake {
  id: string;
  plugin_name: string;
  version: string;
  state: "quarantined" | "trial-ready" | "decided";
  manifest_revision: number;
  correlation_id: string;
}

export interface Trial {
  id: string;
  intake_id: string;
  state: TrialState;
  useful_work_score: number;
  risk_score: number;
  effect_count: number;
  graph: CapabilityEdge[];
  upstreams: UpstreamEvidence[];
}

export interface AdmissionRequest {
  decision: Decision;
  rationale: string;
  trial_id: string;
}

export interface Admission {
  id: string;
  intake_id: string;
  plugin_name: string;
  version: string;
  decision: Decision;
  correlation_id: string;
  receipt_digest: string;
  current_pointer_written: boolean;
  created_at: string;
  trial: Trial;
}

export interface AuditEvent {
  sequence: number;
  kind: string;
  subject: string;
  correlation_id: string;
  occurred_at: string;
  detail: string;
}

export interface RegistryEntry {
  plugin_name: string;
  version: string;
  admission_id: string;
  correlation_id: string;
  receipt_digest: string;
  active: boolean;
  created_at: string;
}

export interface ErrorEnvelope {
  code: string;
  message: string;
  correlation_id: string;
  details?: Record<string, string>;
}

export interface ClientOptions {
  baseUrl?: string;
  token?: string;
  fetcher?: typeof fetch;
  onCorrelation?: (id: string) => void;
}

export class ApiError extends Error {
  constructor(public readonly status: number, public readonly envelope: ErrorEnvelope) {
    super(envelope.message);
    this.name = "ApiError";
  }
}

export class CapabilityMeshClient {
  private readonly baseUrl: string;
  private readonly fetcher: typeof fetch;
  private token: string;
  private readonly onCorrelation?: (id: string) => void;

  constructor(options: ClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? "").replace(/\/$/, "");
    this.fetcher = options.fetcher ?? fetch;
    this.token = options.token ?? "";
    this.onCorrelation = options.onCorrelation;
  }

  setToken(token: string): void { this.token = token; }

  private async request<T>(path: string, init: RequestInit = {}): Promise<T> {
    const correlation = crypto.randomUUID();
    const headers = new Headers(init.headers);
    headers.set("Accept", "application/json");
    headers.set("X-Correlation-ID", correlation);
    if (this.token) headers.set("Authorization", `Bearer ${this.token}`);
    if (init.body && !headers.has("Content-Type")) headers.set("Content-Type", "application/json");
    const response = await this.fetcher(`${this.baseUrl}${path}`, { ...init, headers });
    const responseCorrelation = response.headers.get("x-correlation-id") ?? correlation;
    this.onCorrelation?.(responseCorrelation);
    if (!response.ok) {
      const fallback: ErrorEnvelope = { code: "http_error", message: `Request failed (${response.status})`, correlation_id: responseCorrelation };
      const envelope = await response.json().catch(() => fallback) as ErrorEnvelope;
      throw new ApiError(response.status, envelope);
    }
    if (response.status === 204) return undefined as T;
    return response.json() as Promise<T>;
  }

  ready(): Promise<{ status: string }> { return this.request("/readyz"); }
  createIntake(input: IntakeRequest): Promise<Intake> { return this.request("/api/v1/intakes", { method: "POST", body: JSON.stringify(input) }); }
  getIntake(id: string): Promise<Intake> { return this.request(`/api/v1/intakes/${encodeURIComponent(id)}`); }
  getGraph(id: string): Promise<{ edges: CapabilityEdge[] }> { return this.request(`/api/v1/intakes/${encodeURIComponent(id)}/graph`); }
  startTrial(id: string): Promise<Trial> { return this.request(`/api/v1/intakes/${encodeURIComponent(id)}/trials`, { method: "POST", body: JSON.stringify({ scenario_pack: "frozen-admission-v1" }) }); }
  getTrial(id: string): Promise<Trial> { return this.request(`/api/v1/trials/${encodeURIComponent(id)}`); }
  decide(id: string, input: AdmissionRequest, idempotencyKey: string): Promise<Admission> {
    return this.request(`/api/v1/intakes/${encodeURIComponent(id)}/admissions`, { method: "POST", headers: { "Idempotency-Key": idempotencyKey }, body: JSON.stringify(input) });
  }
  getRegistry(): Promise<{ entries: RegistryEntry[] }> { return this.request("/api/v1/registry"); }
  getRegistryEntry(plugin: string, version: string): Promise<Admission> { return this.request(`/api/v1/registry/${encodeURIComponent(plugin)}/${encodeURIComponent(version)}`); }
  getAudit(correlation: string): Promise<{ events: AuditEvent[] }> { return this.request(`/api/v1/audit/${encodeURIComponent(correlation)}`); }
}
