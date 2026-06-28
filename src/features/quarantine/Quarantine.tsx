import { useEffect, useState } from "react";
import { ErrorBanner, Loading } from "../../components/States";
import { useQuarantineStore } from "../../stores/quarantineStore";
import type { QuarantineRecord } from "../../types/ipc";
import "./quarantine.css";

function levelTone(l: string): "danger" | "warn" | "info" {
  return l === "critical" || l === "high" ? "danger" : l === "medium" ? "warn" : "info";
}
function fileName(p: string) { return p.split(/[\\/]/).pop() ?? p; }
function fmtBytes(n: number) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

/** Quarantine vault — live records from AegisService. */
export function Quarantine() {
  const { items, loading, error, load, restore, remove } = useQuarantineStore();
  const [meta, setMeta] = useState<QuarantineRecord | null>(null);

  useEffect(() => { void load(); }, [load]);

  const active = items.filter((r) => r.status !== "deleted" && r.status !== "restored");
  const totalBytes = active.reduce((s, r) => s + r.size, 0);

  const onDelete = (r: QuarantineRecord) => {
    if (window.confirm("Permanently shred this file? This cannot be undone.")) void remove(r.id);
  };

  return (
    <div className="quarantine-screen">
      {error ? <div className="mb-16"><ErrorBanner error={error} onRetry={() => void load()} /></div> : null}

      <div className="vault-banner rise">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7"><rect x="3" y="11" width="18" height="10" rx="2" /><path d="M7 11V7a5 5 0 0 1 10 0v4" /></svg>
        <div style={{ flex: 1 }}>
          <div className="vt">Encrypted vault — {active.length} item{active.length === 1 ? "" : "s"} isolated</div>
          <div className="vs">Files are AES-256-GCM encrypted at rest; originals neutralised. Quarantined files cannot execute.</div>
        </div>
        <span className="pill ok"><span className="dot" />vault sealed</span>
      </div>

      <div className="grid g-3 mb-16 rise-2">
        <div className="card stat"><div className="s-label">Items in vault</div><div className="s-val">{active.length}</div><div className="s-delta">{fmtBytes(totalBytes)} encrypted</div></div>
        <div className="card stat"><div className="s-label">Total records</div><div className="s-val" style={{ fontSize: 22 }}>{items.length}</div><div className="s-delta">incl. restored/deleted</div></div>
        <div className="card stat"><div className="s-label">Restored</div><div className="s-val" style={{ fontSize: 22 }}>{items.filter((r) => r.status === "restored").length}</div><div className="s-delta">returned to disk</div></div>
      </div>

      <div className="card pad-lg rise-2">
        <div className="card-h"><h3>Quarantined items</h3><span className="ch-sub">restore, delete, or view metadata</span></div>
        {loading ? <Loading label="Loading vault…" /> : active.length === 0 ? (
          <div className="muted" style={{ fontSize: 13, padding: 14 }}>Vault is empty.</div>
        ) : (
          <table className="table">
            <thead><tr><th>File</th><th>Threat level</th><th>Quarantined</th><th>Status</th><th style={{ textAlign: "right" }}>Actions</th></tr></thead>
            <tbody>
              {active.map((r) => (
                <tr key={r.id}>
                  <td onClick={() => setMeta(r)}><div className="cell-stack"><span className="q-name">{fileName(r.original_path)}</span><span className="q-sub">{r.original_path} · {fmtBytes(r.size)}</span></div></td>
                  <td><span className={"pill " + levelTone(r.threat_level)} style={{ fontSize: 10 }}><span className="dot" />{r.threat_level}</span></td>
                  <td className="mono sub">{r.timestamp.slice(0, 10)}</td>
                  <td><span className="lock"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="11" width="18" height="10" rx="2" /><path d="M7 11V7a5 5 0 0 1 10 0v4" /></svg>{r.encrypted ? "encrypted" : "plain"}</span></td>
                  <td style={{ textAlign: "right" }}>
                    <div className="flex gap-8" style={{ justifyContent: "flex-end" }}>
                      <button className="btn ghost sm" onClick={() => setMeta(r)}>Details</button>
                      <button className="btn ghost sm" onClick={() => void restore(r.id)}>Restore</button>
                      <button className="btn danger sm" onClick={() => onDelete(r)}>Delete</button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {meta ? (
        <div className="card pad-lg mt-16">
          <div className="card-h"><h3>Metadata · {fileName(meta.original_path)}</h3><span className="ch-right"><button className="btn ghost sm" onClick={() => setMeta(null)}>Close</button></span></div>
          <dl className="kv">
            <dt>Original path</dt><dd>{meta.original_path}</dd>
            <dt>Vault path</dt><dd>{meta.quarantine_path}</dd>
            <dt>SHA-256</dt><dd>{meta.sha256}</dd>
            <dt>Threat level</dt><dd>{meta.threat_level}</dd>
            <dt>Reason</dt><dd>{meta.reason}</dd>
            <dt>Size</dt><dd>{fmtBytes(meta.size)}</dd>
            <dt>Quarantined</dt><dd>{meta.timestamp}</dd>
            <dt>Status</dt><dd>{meta.status}</dd>
          </dl>
        </div>
      ) : null}
    </div>
  );
}
