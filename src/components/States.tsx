import { inTauri } from "../lib/ipc";

/** Human hint for a raw service error message. */
function classify(error: string): string {
  const e = error.toLowerCase();
  if (!inTauri()) return "AegisService is only available in the desktop app (running in browser preview).";
  if (e.includes("not configured")) return "This subsystem is not configured yet.";
  if (e.includes("database")) return "Database unavailable — the service could not open its store.";
  if (e.includes("unavailable") || e.includes("service")) return "AegisService is unavailable.";
  if (e.includes("network") || e.includes("http") || e.includes("download")) return "Network/download failure.";
  return "The service reported an error.";
}

export function ErrorBanner({ error, onRetry }: { error: string; onRetry?: () => void }) {
  return (
    <div className="card pad-lg" style={{ borderColor: "color-mix(in srgb,var(--warn) 40%,transparent)" }} role="alert">
      <div className="flex items-center gap-12">
        <svg viewBox="0 0 24 24" fill="none" stroke="var(--warn)" strokeWidth="1.8" style={{ width: 20, height: 20, flex: "none" }}>
          <path d="M12 9v4M12 17h.01M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z" />
        </svg>
        <div style={{ flex: 1 }}>
          <div style={{ fontWeight: 600, fontSize: 13.5, color: "var(--warn)" }}>{classify(error)}</div>
          <div className="mono" style={{ fontSize: 11.5, color: "var(--muted)", marginTop: 3 }}>{error}</div>
        </div>
        {onRetry ? (
          <button className="btn ghost sm" onClick={onRetry}>Retry</button>
        ) : null}
      </div>
    </div>
  );
}

export function Loading({ label = "Loading…" }: { label?: string }) {
  return (
    <div className="card pad-lg" style={{ color: "var(--muted)", fontSize: 13 }}>
      {label}
    </div>
  );
}

export function Empty({ label }: { label: string }) {
  return (
    <div className="card pad-lg" style={{ color: "var(--muted)", fontSize: 13, textAlign: "center" }}>
      {label}
    </div>
  );
}
