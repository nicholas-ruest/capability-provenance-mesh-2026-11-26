import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App";
import type { CapabilityMeshClient } from "../../api/client";

function client(): CapabilityMeshClient {
  return {
    setToken: vi.fn(),
    ready: vi.fn().mockResolvedValue({ status: "ready" }),
    getRegistry: vi.fn().mockResolvedValue({ entries: [] }),
  } as unknown as CapabilityMeshClient;
}

describe("Capability Provenance Mesh", () => {
  it("renders its domain workbench and loading-backed readiness", async () => {
    render(<App client={client()} />);
    expect(screen.getByRole("heading", { name: "Admission" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Quarantine and run trial" })).toBeEnabled();
    expect(screen.getByText("Evidence closes here")).toBeInTheDocument();
    expect(await screen.findByText("Service ready")).toBeInTheDocument();
  });

  it("exposes every product section through semantic navigation", () => {
    render(<App client={client()} />);
    const nav = screen.getByRole("navigation", { name: "Product sections" });
    expect(nav).toHaveTextContent("Registry");
    expect(nav).toHaveTextContent("Audit replay");
    expect(nav).toHaveTextContent("Evidence mesh");
  });
});
