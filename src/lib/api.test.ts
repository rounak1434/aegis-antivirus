import { describe, it, expect, vi, beforeEach } from "vitest";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import { api } from "./api";

describe("typed IPC api", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue([]);
  });

  it("scan.start forwards mode + roots", async () => {
    await api.scan.start("quick", ["C:\\Users"]);
    expect(invokeMock).toHaveBeenCalledWith("start_scan", { mode: "quick", roots: ["C:\\Users"] });
  });

  it("scan.stop forwards jobId (camelCase)", async () => {
    invokeMock.mockResolvedValue(true);
    await api.scan.stop("job-1");
    expect(invokeMock).toHaveBeenCalledWith("stop_scan", { jobId: "job-1" });
  });

  it("quarantine.restore defaults dest to null", async () => {
    invokeMock.mockResolvedValue("C:\\path");
    await api.quarantine.restore("id-1");
    expect(invokeMock).toHaveBeenCalledWith("restore_file", { id: "id-1", dest: null });
  });

  it("settings.save wraps json under settings key", async () => {
    invokeMock.mockResolvedValue(undefined);
    await api.settings.save('{"a":1}');
    expect(invokeMock).toHaveBeenCalledWith("save_settings", { settings: '{"a":1}' });
  });

  it("normalizes errors to ServiceError", async () => {
    invokeMock.mockRejectedValue(new Error("engine down"));
    await expect(api.health.get()).rejects.toMatchObject({
      name: "ServiceError",
      command: "get_service_health",
      message: "engine down",
    });
  });
});
