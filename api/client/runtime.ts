import { CapabilitySubmission, DecisionRecord, ErrorEnvelope } from "./schema";
export type { CapabilityManifest, CapabilitySubmission, DecisionRecord } from "./schema";

export interface UpstreamEvidence {
  project: string;
  revision: string;
  responsibility: string;
  status: "verified" | "failed";
}

export class ApiError extends Error {
  constructor(public readonly status: number, public readonly envelope: ErrorEnvelope) {
    super(envelope.message);
    this.name = "ApiError";
  }
}

export interface RuntimeClientOptions {
  baseUrl?: string;
  token?: string;
  fetcher?: typeof fetch;
}

const responsibilities: Record<string, string> = {
  "ruvnet/autogenous": "capability protocol derivation",
  "ruvnet/ruGraph": "typed provenance graph",
  "ruvnet/agenticow": "authority boundary analysis",
  "ruvnet/ai-text-watermark": "receipt watermark verification",
  "ruvnet/LatentMesh": "signed observation envelopes",
  "NVIDIA/SkillSpector": "static capability inspection",
  "sigstore/cosign": "artifact attestation",
};

export function evidenceFrom(record: DecisionRecord): UpstreamEvidence[] {
  return record.upstream_receipts.map((receipt) => {
    const [identity] = receipt.split(":");
    const separator = identity.lastIndexOf("@");
    const project = separator < 0 ? identity : identity.slice(0, separator);
    const revision = separator < 0 ? "unknown" : identity.slice(separator + 1);
    const canonical = Object.keys(responsibilities).find((key) => key.toLowerCase() === project.toLowerCase()) ?? project;
    return {
      project: canonical,
      revision,
      responsibility: responsibilities[canonical] ?? "executed product responsibility",
      status: "verified",
    };
  });
}

export class RuntimeClient {
  private readonly baseUrl: string;
  private readonly fetcher: typeof fetch;
  private token: string;

  constructor(options: RuntimeClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? "").replace(/\/$/, "");
    this.fetcher =
      options.fetcher ?? ((input, init) => globalThis.fetch(input, init));
    this.token = options.token ?? "";
  }

  setToken(token: string): void { this.token = token; }

  private async request<T>(path: string, init: RequestInit = {}): Promise<T> {
    const correlation = crypto.randomUUID();
    const headers = new Headers(init.headers);
    headers.set("Accept", "application/json");
    headers.set("X-Correlation-ID", correlation);
    headers.set("X-CPM-Role", "reviewer");
    if (this.token) headers.set("Authorization", `Bearer ${this.token}`);
    if (init.body) headers.set("Content-Type", "application/json");
    const response = await this.fetcher(`${this.baseUrl}${path}`, { ...init, headers });
    const responseCorrelation = response.headers.get("x-correlation-id") ?? correlation;
    if (!response.ok) {
      const fallback: ErrorEnvelope = { code: "http_error", message: `Request failed (${response.status})`, correlation_id: responseCorrelation };
      const envelope = await response.json().catch(() => fallback) as ErrorEnvelope;
      throw new ApiError(response.status, envelope);
    }
    if (response.headers.get("content-length") === "0") return { status: "ready" } as T;
    const text = await response.text();
    return (text ? JSON.parse(text) : { status: "ready" }) as T;
  }

  ready(): Promise<{ status: string }> { return this.request("/readyz"); }

  submit(input: CapabilitySubmission, policy: "allow" | "deny" = "allow"): Promise<DecisionRecord> {
    const headers = policy === "deny" ? { "X-CPM-Policy": "deny" } : undefined;
    return this.request("/api/v1/intakes", { method: "POST", headers, body: JSON.stringify(input) });
  }

  getDecision(plugin: string, version: string): Promise<DecisionRecord> {
    return this.request(`/api/v1/registry/${encodeURIComponent(plugin)}/${encodeURIComponent(version)}`);
  }

  getAudit(correlation: string): Promise<DecisionRecord> {
    return this.request(`/api/v1/audit/${encodeURIComponent(correlation)}`);
  }
}
