import { useEffect, useMemo, useState } from "react";
import { ErrorBanner, Loading } from "../../components/States";
import { useThreatStore } from "../../stores/threatStore";
import type { ThreatDetection, ThreatEvidence, ThreatLevel } from "../../types/ipc";
import "./threats.css";

const LEVELS: (ThreatLevel | "all")[] = ["all", "critical", "high", "medium", "low"];

function levelTone(l: ThreatLevel): "danger" | "warn" | "info" {
  return l === "critical" || l === "high" ? "danger" : l === "medium" ? "warn" : "info";
}
function scoreColor(n: number) { return n >= 85 ? "var(--danger)" : n >= 60 ? "var(--warn)" : "var(--info)"; }

/** Render an evidence item as readable text (mirrors the Rust reason()). */
function evidenceText(e: ThreatEvidence): string {
  const f = e as Record<string, unknown>;
  const ent = (v: unknown) => (typeof v === "number" ? v.toFixed(2) : String(v));
  switch (e.kind) {
    case "hash_match": return `Hash match (${f.algo}): ${f.hex}`;
    case "yara_match": return `YARA ${f.namespace}:${f.rule}`;
    case "entropy_detection": return `High entropy ${ent(f.entropy)}`;
    case "packed_executable": return `Packed executable (entropy ${ent(f.entropy)})`;
    case "double_extension": return `Double extension ${f.file_name}`;
    case "suspicious_extension": return `Suspicious extension .${f.ext}`;
    case "script_indicator": return `Script indicator: ${f.indicator}`;
    case "powershell_indicator": return `PowerShell abuse: ${f.indicator}`;
    case "persistence_mechanism": return `${f.mechanism} '${f.name}'`;
    case "suspicious_location": return `${f.path} (${f.reason})`;
    default: return e.kind;
  }
}

function fileName(p: string) { return p.split(/[\\/]/).pop() ?? p; }

export function ThreatCenter() {
  const { items, loading, error, load, quarantine } = useThreatStore();
  const [filter, setFilter] = useState<ThreatLevel | "all">("all");
  const [search, setSearch] = useState("");
  const [sortDesc, setSortDesc] = useState(true);
  const [selected, setSelected] = useState<ThreatDetection | null>(null);

  useEffect(() => { void load(); }, [load]);

  const rows = useMemo(() => {
    let r = items;
    if (filter !== "all") r = r.filter((t) => t.threat_level === filter);
    if (search.trim()) r = r.filter((t) => t.path.toLowerCase().includes(search.toLowerCase()));
    return [...r].sort((a, b) => (sortDesc ? b.score - a.score : a.score - b.score));
  }, [items, filter, search, sortDesc]);

  const counts = useMemo(() => {
    const c: Record<string, number> = { all: items.length };
    for (const t of items) c[t.threat_level] = (c[t.threat_level] ?? 0) + 1;
    return c;
  }, [items]);

  return (
    <div className="threat-screen">
      {error ? <div className="mb-16"><ErrorBanner error={error} onRetry={() => void load()} /></div> : null}

      <div className="grid g-4 mb-16 rise">
        <StatTile label="Active threats" value={items.length} delta="from scans" />
        <StatTile label="Critical/High" value={(counts.critical ?? 0) + (counts.high ?? 0)} tone="danger" delta="quarantine recommended" />
        <StatTile label="Medium" value={counts.medium ?? 0} tone="warn" delta="review" />
        <StatTile label="Low" value={counts.low ?? 0} delta="informational" />
      </div>

      <div className="card pad-lg rise-2">
        <div className="card-h">
          <h3>Detections</h3>
          <span className="ch-sub">click a row for evidence</span>
          <div className="ch-right">
            <input className="sel-input" placeholder="Search path…" value={search} onChange={(e) => setSearch(e.target.value)} style={{ marginLeft: 0 }} aria-label="Search threats" />
            <button className="btn ghost sm" onClick={() => setSortDesc((v) => !v)}>Score {sortDesc ? "↓" : "↑"}</button>
          </div>
        </div>
        <div className="filters">
          {LEVELS.map((l) => (
            <span key={l} className={"chip" + (filter === l ? " on" : "")} onClick={() => setFilter(l)} role="button" tabIndex={0}>
              {l[0].toUpperCase() + l.slice(1)} <span className="c">{counts[l] ?? 0}</span>
            </span>
          ))}
        </div>

        {loading ? <Loading label="Loading threats…" /> : rows.length === 0 ? (
          <div className="muted" style={{ fontSize: 13, padding: 14 }}>No threats match.</div>
        ) : (
          <table className="table">
            <thead><tr><th>Threat</th><th>File</th><th>Evidence</th><th>Risk</th></tr></thead>
            <tbody>
              {rows.map((t) => (
                <tr key={t.id} onClick={() => setSelected(t)}>
                  <td><div className="name-cell"><span className="tn">{t.threat_level}</span><span className="tr">{t.evidence.length} signals</span></div></td>
                  <td><div className="cell-stack"><span className="path">{fileName(t.path)}</span><span className="sub">{t.path}</span></div></td>
                  <td><span className={"pill " + levelTone(t.threat_level)}><span className="dot" />{t.evidence[0] ? String((t.evidence[0] as Record<string, unknown>).kind) : "—"}</span></td>
                  <td><span className="sev"><span className="bar"><i style={{ width: t.score + "%", background: scoreColor(t.score) }} /></span>{t.score}</span></td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <div className={"drawer-back" + (selected ? " open" : "")} onClick={() => setSelected(null)} />
      <aside className={"drawer" + (selected ? " open" : "")}>
        {selected ? (
          <>
            <div className="drawer-h">
              <button className="icon-btn x" onClick={() => setSelected(null)} aria-label="Close">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8"><line x1="6" y1="6" x2="18" y2="18" /><line x1="18" y1="6" x2="6" y2="18" /></svg>
              </button>
              <span className="eyebrow">Threat detail</span>
              <h2 style={{ fontSize: 18, marginTop: 6 }}>{fileName(selected.path)}</h2>
              <div className="flex gap-8 mt-8"><span className={"pill " + levelTone(selected.threat_level)}><span className="dot" />{selected.threat_level}</span><span className="pill muted">score {selected.score}</span></div>
            </div>
            <div className="drawer-b">
              <div className="big-risk">
                <div className="rn" style={{ color: scoreColor(selected.score) }}>{selected.score}</div>
                <div className="flex col gap-8" style={{ flex: 1 }}>
                  <div className="muted" style={{ fontSize: 12 }}>Aggregated risk score</div>
                  <div className="bar-track"><div className="bar-fill" style={{ width: selected.score + "%", background: scoreColor(selected.score) }} /></div>
                </div>
              </div>
              <div className="ev mt-16">
                <div className="et">File</div>
                <dl className="kv"><dt>Path</dt><dd>{selected.path}</dd><dt>Detected</dt><dd>{selected.timestamp}</dd></dl>
              </div>
              <div className="ev">
                <div className="et">Evidence ({selected.evidence.length})</div>
                {selected.evidence.map((e, i) => (
                  <div className="layer-hit" key={i}><span className="pill muted" style={{ fontSize: 10 }}><span className="dot" />{e.kind}</span><span style={{ color: "var(--fg-2)" }}>{evidenceText(e)}</span></div>
                ))}
              </div>
            </div>
            <div className="drawer-f">
              <button className="btn primary" style={{ flex: 1 }} onClick={() => { void quarantine(selected); setSelected(null); }}>Quarantine</button>
            </div>
          </>
        ) : null}
      </aside>
    </div>
  );
}

function StatTile({ label, value, tone, delta }: { label: string; value: number; tone?: "danger" | "warn"; delta: string }) {
  return (
    <div className="card stat">
      <div className="s-label">{label}</div>
      <div className={"s-val" + (tone ? " text-" + tone : "")} style={{ fontSize: 22 }}>{value}</div>
      <div className="s-delta">{delta}</div>
    </div>
  );
}
