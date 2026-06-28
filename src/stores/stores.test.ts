import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("../lib/api", () => ({
  api: {
    threats: { list: vi.fn(), quarantine: vi.fn() },
    scan: { start: vi.fn(), stop: vi.fn(), status: vi.fn(), jobs: vi.fn() },
    quarantine: { list: vi.fn(), restore: vi.fn(), delete: vi.fn() },
    realtime: { status: vi.fn(), start: vi.fn(), stop: vi.fn() },
  },
}));

import { api } from "../lib/api";
import { useThreatStore } from "./threatStore";
import { useScanStore } from "./scanStore";
import { useRealtimeStore } from "./realtimeStore";

const detection = {
  id: "d1",
  path: "C:\\x.exe",
  threat_level: "critical" as const,
  score: 100,
  evidence: [],
  timestamp: "2026-06-24T00:00:00Z",
};

describe("threatStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useThreatStore.setState({ items: [], loading: false, error: null });
  });

  it("load populates items", async () => {
    (api.threats.list as ReturnType<typeof vi.fn>).mockResolvedValue([detection]);
    await useThreatStore.getState().load();
    expect(useThreatStore.getState().items).toHaveLength(1);
    expect(useThreatStore.getState().error).toBeNull();
  });

  it("load failure sets error and clears items", async () => {
    (api.threats.list as ReturnType<typeof vi.fn>).mockRejectedValue(new Error("db unavailable"));
    await useThreatStore.getState().load();
    expect(useThreatStore.getState().items).toEqual([]);
    expect(useThreatStore.getState().error).toBe("db unavailable");
  });
});

describe("scanStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useScanStore.setState({ activeJobId: null, job: null, jobs: [], error: null });
  });

  it("start sets active job id and polls", async () => {
    (api.scan.start as ReturnType<typeof vi.fn>).mockResolvedValue("job-9");
    (api.scan.status as ReturnType<typeof vi.fn>).mockResolvedValue({ id: "job-9", status: "running" });
    await useScanStore.getState().start("full", ["C:\\"]);
    expect(useScanStore.getState().activeJobId).toBe("job-9");
    expect(api.scan.start).toHaveBeenCalledWith("full", ["C:\\"]);
    expect(api.scan.status).toHaveBeenCalledWith("job-9");
  });
});

describe("realtimeStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useRealtimeStore.setState({ status: null, error: null });
  });

  it("start re-loads status", async () => {
    (api.realtime.start as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);
    (api.realtime.status as ReturnType<typeof vi.fn>).mockResolvedValue({
      running: true,
      mode: "auto_quarantine",
      watched_paths: ["C:\\Users"],
      events_processed: 0,
      alerts_raised: 0,
    });
    await useRealtimeStore.getState().start("auto_quarantine");
    expect(api.realtime.start).toHaveBeenCalledWith("auto_quarantine");
    expect(useRealtimeStore.getState().status?.running).toBe(true);
  });
});
