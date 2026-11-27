import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ProductApp } from "./ProductApp";
import type { RuntimeClient } from "../../api/client/runtime";

afterEach(cleanup);

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
});
