import { useEffect, useState } from "react";
import { Icon } from "../../components/Icon";
import { THREATS, sevPill, sevColor } from "./data";
import type { ThreatRow, ThreatSeverity } from "./data";
import "./threats.css";

type Filter = "all" | ThreatSeverity;

const FILTERS: { key: Filter; label: string; count: number }[] = [
  { key: "all", label: "All", count: THREATS.length },
  { key: "malicious", label: "Malicious", count: THREATS.filter((t) => t.severity === "malicious").length },
  { key: "suspicious", label: "Suspicious", count: THREATS.filter((t) => t.severity === "suspicious").length },
  { key: "pua", label: "PUA", count: THREATS.filter((t) => t.severity === "pua").length }
];

/** Threat Center — React conversion of design-prototype/threats.html. */
export function ThreatCenter() {
  const [filter, setFilter] = useState<Filter>("all");
  const [selected, setSelected] = useState<ThreatRow | null>(null);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setSelected(null);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const rows = THREATS.filter((t) => filter === "all" || t.severity === filter);

  return (
    <div className="threats-screen">
      <div className="grid g-4 mb-16 rise">
        <StatCard label="Active threats" value="3" tone="danger" delta="from deep scan · 4h ago" />
        <StatCard label="Malicious" value="2" tone="danger" small delta="recommend quarantine" />
        <StatCard label="Suspicious" value="1" tone="warn" small delta="review required" />
        <StatCard label="Resolved (7d)" value="14" small delta="all quarantined/removed" deltaUp />
      </div>

      <div className="card pad-lg rise-2">
        <div className="card-h">
          <h3>Detections</h3>
          <span className="ch-sub">click a row for evidence &amp; recommended action</span>
          <div className="ch-right">
            <button className="btn ghost sm">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" style={{ width: 15, height: 15 }}>
                <path d="M12 3v12M7 10l5 5 5-5M5 21h14" />
              </svg>
              Export report
            </button>
            <button className="btn primary sm">
              <Icon name="box" strokeWidth={1.8} />
              Quarantine all malicious
            </button>
          </div>
        </div>

        <div className="filters">
          {FILTERS.map((f) => (
            <span key={f.key} className={"chip" + (filter === f.key ? " on" : "")} onClick={() => setFilter(f.key)}>
              {f.label} <span className="c">{f.count}</span>
            </span>
          ))}
        </div>

        <table className="table">
          <thead>
            <tr>
              <th>Threat</th>
              <th>File</th>
              <th>Layer</th>
              <th>Risk</th>
              <th>Recommended</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.name} onClick={() => setSelected(r)}>
                <td>
                  <div className="name-cell">
                    <span className="tn">{r.name}</span>
                    <span className="tr">{r.family} family</span>
                  </div>
                </td>
                <td>
                  <div className="cell-stack">
                    <span className="path">{r.file}</span>
                    <span className="sub">{r.path.replace(r.file, "").replace(" › ", "")}</span>
                  </div>
                </td>
                <td>
                  <span className={"pill " + sevPill(r.severity)}>
                    <span className="dot" />
                    {r.severity}
                  </span>
                </td>
                <td>
                  <span className="sev">
                    <span className="bar">
                      <i style={{ width: r.score + "%", background: sevColor(r.score) }} />
                    </span>
                    {r.score}
                  </span>
                </td>
                <td>
                  <span style={{ fontSize: 12, color: r.severity === "malicious" ? "var(--danger)" : "var(--warn)" }}>
                    {r.severity === "malicious" ? "Quarantine" : "Review"}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className={"drawer-back" + (selected ? " open" : "")} onClick={() => setSelected(null)} />
      <ThreatDrawer threat={selected} onClose={() => setSelected(null)} />
    </div>
  );
}

function StatCard({ label, value, tone, small, delta, deltaUp }: { label: string; value: string; tone?: "danger" | "warn"; small?: boolean; delta: string; deltaUp?: boolean }) {
  return (
    <div className="card stat">
      <div className="s-label">{label}</div>
      <div className={"s-val" + (tone ? " text-" + tone : "")} style={small ? { fontSize: 22 } : undefined}>
        {value}
      </div>
      <div className={"s-delta" + (deltaUp ? " up" : "")}>{delta}</div>
    </div>
  );
}

function ThreatDrawer({ threat, onClose }: { threat: ThreatRow | null; onClose: () => void }) {
  const open = threat !== null;
  return (
    <aside className={"drawer" + (open ? " open" : "")} aria-hidden={!open}>
      <div className="drawer-h">
        <button className="icon-btn x" onClick={onClose} aria-label="Close">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" style={{ width: 16, height: 16 }}>
            <line x1="6" y1="6" x2="18" y2="18" />
            <line x1="18" y1="6" x2="6" y2="18" />
          </svg>
        </button>
        <span className="eyebrow">Threat detail</span>
        <h2 style={{ fontSize: 18, marginTop: 6 }}>{threat?.name ?? "—"}</h2>
        <div className="flex gap-8 mt-8">
          <span className={"pill " + (threat ? sevPill(threat.severity) : "danger")}>
            <span className="dot" />
            {threat?.severity ?? "—"}
          </span>
          <span className="pill muted">{threat ? threat.family + " family" : "—"}</span>
        </div>
      </div>
      {threat ? (
        <div className="drawer-b">
          <div className="big-risk">
            <div className="rn" style={{ color: sevColor(threat.score) }}>{threat.score}</div>
            <div className="flex col gap-8" style={{ flex: 1 }}>
              <div className="muted" style={{ fontSize: 12 }}>Aggregated risk score</div>
              <div className="bar-track">
                <div className="bar-fill danger" style={{ width: threat.score + "%", background: sevColor(threat.score) }} />
              </div>
            </div>
          </div>

          <div className="recommend">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8">
              <path d="M12 9v4M12 17h.01M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z" />
            </svg>
            <span>{threat.rec}</span>
          </div>

          <div className="ev mt-16">
            <div className="et">File</div>
            <dl className="kv">
              <dt>Path</dt><dd>{threat.path}</dd>
              <dt>SHA-256</dt><dd>{threat.sha}</dd>
              <dt>Size</dt><dd>{threat.size}</dd>
              <dt>Signed</dt><dd>{threat.signed}</dd>
            </dl>
          </div>

          <div className="ev">
            <div className="et">Detection layers that fired</div>
            {threat.layers.map((l) => (
              <div className="layer-hit" key={l.layer}>
                <span className={"pill " + l.tone} style={{ fontSize: 10 }}>
                  <span className="dot" />
                  {l.layer}
                </span>
                <span style={{ color: "var(--fg-2)" }}>{l.detail}</span>
              </div>
            ))}
          </div>

          <div className="ev">
            <div className="et">Why it was flagged</div>
            <div style={{ fontSize: 12.5, color: "var(--fg-2)", lineHeight: 1.55 }}>{threat.reasons}</div>
          </div>
        </div>
      ) : (
        <div className="drawer-b" />
      )}
      <div className="drawer-f">
        <button className="btn primary" style={{ flex: 1 }}>
          <Icon name="box" strokeWidth={1.8} />
          Quarantine
        </button>
        <button className="btn ghost">Allow once</button>
        <button className="btn danger">Delete</button>
      </div>
    </aside>
  );
}
