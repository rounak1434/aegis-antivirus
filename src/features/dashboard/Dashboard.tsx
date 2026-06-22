import { useEffect } from "react";
import { useSecurityStore } from "../../stores/securityStore";

export function Dashboard() {
  const { status, statusError, activeScanId, refreshStatus, requestScan } = useSecurityStore();

  useEffect(() => {
    void refreshStatus();
  }, [refreshStatus]);

  return (
    <div className="space-y-6">
      <section className="aegis-card p-6">
        <div className="flex items-start justify-between gap-6">
          <div>
            <div className="mb-2 text-xs font-semibold uppercase tracking-[0.08em] text-accent">Service boundary</div>
            <h2 className="text-3xl font-semibold tracking-[-0.02em]">AegisService owns protection</h2>
            <p className="mt-3 max-w-2xl text-sm leading-6 text-muted">
              The desktop app is intentionally UI-only. Real-time protection, monitoring, scheduled scans, quarantine, and updates are routed to the Windows service through typed IPC commands.
            </p>
          </div>
          <button className="aegis-button" onClick={() => void refreshStatus()}>Refresh service status</button>
        </div>
      </section>

      <section className="grid grid-cols-4 gap-4">
        <StatusCard label="Service" value={status?.health ?? "unknown"} />
        <StatusCard label="Real-time" value={status?.realTimeProtection ? "on" : "off"} />
        <StatusCard label="File monitor" value={status?.fileMonitor ? "on" : "off"} />
        <StatusCard label="Process monitor" value={status?.processMonitor ? "on" : "off"} />
      </section>

      {statusError ? (
        <section className="aegis-card border-warn/60 p-5 text-sm text-warn">
          Service status could not be read: {statusError}
        </section>
      ) : null}

      <section className="aegis-card p-5">
        <div className="mb-4 flex items-center justify-between">
          <h3 className="text-sm font-semibold">Scan commands</h3>
          {activeScanId ? <span className="font-mono text-xs text-muted">active scan {activeScanId}</span> : null}
        </div>
        <div className="flex flex-wrap gap-3">
          <button className="aegis-button aegis-button-primary" onClick={() => void requestScan("quick")}>Quick scan</button>
          <button className="aegis-button" onClick={() => void requestScan("full")}>Full scan</button>
          <button className="aegis-button" onClick={() => void requestScan("deep")}>Deep scan</button>
          <button className="aegis-button" onClick={() => void requestScan("custom")}>Custom scan</button>
        </div>
      </section>
    </div>
  );
}

function StatusCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="aegis-card p-4">
      <div className="text-xs uppercase tracking-[0.08em] text-muted">{label}</div>
      <div className="mt-2 font-mono text-xl text-fg">{value}</div>
    </div>
  );
}
