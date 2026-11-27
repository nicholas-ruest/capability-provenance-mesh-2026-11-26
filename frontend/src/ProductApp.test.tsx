import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ProductApp } from "./ProductApp";
import type { RuntimeClient } from "../../api/client/runtime";

afterEach(() => { cleanup(); localStorage.clear(); });

function client(): RuntimeClient {
  return {
    setToken: vi.fn(),
    ready: vi.fn().mockResolvedValue({ status: "ready" }),
    getDecision: vi.fn().mockRejectedValue(new Error("none")),
  } as unknown as RuntimeClient;
}

describe("real-stack product shell", () => {
  it("renders admission controls and readiness states", async () => {
    render(<ProductApp suppliedClient={client()} />);
    expect(screen.getByRole("button", { name: "Run governed admission" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Run policy-denial probe" })).toBeEnabled();
    expect(await screen.findByText("Service ready")).toBeInTheDocument();
  });

  it("provides semantic navigation for durable workflows", () => {
    render(<ProductApp suppliedClient={client()} />);
    expect(screen.getByRole("navigation", { name: "Product sections" })).toHaveTextContent("Audit replay");
  });

  it("persists the submitted plugin and version for refresh retrieval", async () => {
    const supplied = client();
    supplied.submit = vi.fn().mockResolvedValue({
      correlation_id: "8dca901e-b6ac-4c5d-b2dc-ef94df63ac12", admitted: true,
      artifact_digest: "a".repeat(64), upstream_receipts: Array(7).fill("receipt"),
      receipt_checksum: "b".repeat(64),
    });
    render(<ProductApp suppliedClient={supplied} />);
    fireEvent.click(screen.getByRole("button", { name: "Run governed admission" }));
    await waitFor(() => expect(localStorage.getItem("cpm:last-decision")).toBe(
      JSON.stringify({ plugin: "image-transform", version: "1.0.0" }),
    ));
  });
});
