import { useEffect } from "react";
import { ErrorBanner, Loading } from "../../components/States";
import { useUpdateStore } from "../../stores/updateStore";

const LABELS: Record<string, string> = {
  signature_database: "Signature database",
  yara_rules: "YARA rules",
  threat_metadata: "Threat metadata",
  engine_config: "Engine configuration"
};

/** Update center — installed components, availability check, install, rollback. */
export function Updates() {
  const { installed, available, loading, checking, installing, error, lastAction, load, check, rollback } = useUpdateStore();

  useEffect(() => { void load(); }, [load]);

  const sig = installed.find(([c]) => c === "signature_database");

  return (
    <div className="updates-screen">
      {error ? <div className="mb-16"><ErrorBanner error={error} onRetry={() => void load()} /></div> : null}

      <div className="card pad-lg rise mb-16">
        <div className="flex items-center gap-16">
          <div style={{ flex: 1 }}>
            <div style={{ fontWeight: 600, fontSize: 15 }}>{sig ? `Signatures ${sig[1]}` : "No signatures installed"}</div>
            <div className="muted" style={{ fontSize: 12.5, marginTop: 3 }}>
              {installed.length} component{installed.length === 1 ? "" : "s"} installed · verified by SHA-256 + Ed25519
            </div>
          </div>
          <button className="btn primary" onClick={() => void check([])} disabled={checking}>
            {checking ? "Checking…" : "Check for updates"}
          </button>
        </div>
        {lastAction ? <div className="mt-8" style={{ fontSize: 12, color: "var(--accent)" }}>{lastAction}</div> : null}
        <ul className="secure-list" style={{ listStyle: "none", marginTop: 14, display: "flex", flexDirection: "column", gap: 8 }}>
          <li className="muted" style={{ fontSize: 12.5 }}>✓ Payloads verified against a pinned signing key before install</li>
          <li className="muted" style={{ fontSize: 12.5 }}>✓ Atomic swap with automatic rollback on failure</li>
          <li className="muted" style={{ fontSize: 12.5 }}>✓ Downgrade/rollback attacks rejected</li>
        </ul>
      </div>

      <div className="grid g-2">
        <div className="card pad-lg rise-2">
          <div className="card-h"><h3>Installed components</h3></div>
          {loading ? <Loading label="Loading…" /> : installed.length === 0 ? (
            <div className="muted" style={{ fontSize: 13 }}>Nothing installed yet.</div>
          ) : (
            <table className="table">
              <thead><tr><th>Component</th><th>Version</th><th style={{ textAlign: "right" }}>Actions</th></tr></thead>
              <tbody>
                {installed.map(([component, version]) => (
                  <tr key={component}>
                    <td>{LABELS[component] ?? component}</td>
                    <td className="mono">{version}</td>
                    <td style={{ textAlign: "right" }}>
                      <button className="btn ghost sm" onClick={() => void rollback(component)} disabled={installing === component}>Rollback</button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        <div className="card pad-lg rise-3">
          <div className="card-h"><h3>Available updates</h3><span className="ch-sub">from the configured feed</span></div>
          {installing ? (
            <div className="mb-16">
              <div style={{ fontSize: 12, color: "var(--muted)", marginBottom: 6 }}>Installing {LABELS[installing] ?? installing}…</div>
              <div className="bar-track"><div className="bar-fill" style={{ width: "60%" }} /></div>
            </div>
          ) : null}
          {available.length === 0 ? (
            <div className="muted" style={{ fontSize: 13 }}>{checking ? "Checking…" : "No updates available. (A signed feed must be configured.)"}</div>
          ) : (
            available.map((m) => (
              <div className="surface-row" key={m.component + m.version}>
                <div><div className="sn">{LABELS[m.component] ?? m.component}</div><div className="sd">v{m.version} · {(m.size / 1024).toFixed(0)} KB</div></div>
                <div className="sv"><span className="pill info"><span className="dot" />available</span></div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
