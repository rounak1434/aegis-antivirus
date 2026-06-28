import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";

vi.mock("../../lib/api", () => ({
  api: {
    health: { get: vi.fn().mockResolvedValue({ scanner: "ok", database: "ok", rules: "degraded", quarantine: "ok", active_jobs: 0, overall: "degraded" }) },
    threats: { list: vi.fn().mockResolvedValue([]) },
    quarantine: { list: vi.fn().mockResolvedValue([]) },
    realtime: { status: vi.fn().mockResolvedValue({ running: true, mode: "notify_only", watched_paths: [], events_processed: 5, alerts_raised: 1 }) },
    updates: { status: vi.fn().mockResolvedValue([["signature_database", "2024.06.22.02"]]) },
  },
}));

import { Dashboard } from "./Dashboard";

describe("Dashboard (live)", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders service-driven content (no mock arrays)", async () => {
    render(
      <MemoryRouter>
        <Dashboard />
      </MemoryRouter>
    );
    // Static structure renders immediately…
    expect(screen.getByText("Run a scan")).toBeTruthy();
    // …and live data arrives from the (mocked) service.
    expect(await screen.findByText(/Defs 2024\.06\.22\.02/)).toBeTruthy();
    expect(await screen.findByText("Service health")).toBeTruthy();
  });
});
