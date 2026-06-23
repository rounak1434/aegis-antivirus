import { useState } from "react";
import "./settings.css";

type TabKey = "protection" | "scanning" | "updates" | "exclusions" | "security";

const TABS: { key: TabKey; label: string }[] = [
  { key: "protection", label: "Protection" },
  { key: "scanning", label: "Scanning" },
  { key: "updates", label: "Updates" },
  { key: "exclusions", label: "Exclusions" },
  { key: "security", label: "Security" }
];

/** A controlled toggle switch matching the prototype's `.toggle` element. */
function Toggle({ defaultOn = true, label }: { defaultOn?: boolean; label: string }) {
  const [on, setOn] = useState(defaultOn);
  return (
    <button
      className={"toggle" + (on ? "" : " off")}
      onClick={() => setOn((v) => !v)}
      role="switch"
      aria-checked={on}
      aria-label={label}
    />
  );
}

function Opt({ title, sub, children }: { title: string; sub: string; children: React.ReactNode }) {
  return (
    <div className="opt">
      <div>
        <div className="on-t">{title}</div>
        <div className="on-s">{sub}</div>
      </div>
      {children}
    </div>
  );
}

const UPDATE_STEPS: [number, string][] = [
  [20, "Connecting over TLS 1.3…"],
  [45, "Downloading delta (2,104 signatures)…"],
  [70, "Verifying signature against pinned key…"],
  [90, "Applying atomic swap…"],
  [100, "Up to date · rollback point saved ✓"]
];

/** Settings & updates — React conversion of design-prototype/settings.html. */
export function Settings() {
  const [tab, setTab] = useState<TabKey>("protection");
  const [updProgress, setUpdProgress] = useState<number | null>(null);
  const [updStatus, setUpdStatus] = useState<string>("");
  const [updChecking, setUpdChecking] = useState(false);

  const runUpdate = () => {
    if (updChecking) return;
    setUpdChecking(true);
    setUpdProgress(0);
    let i = 0;
    const iv = setInterval(() => {
      const [pct, msg] = UPDATE_STEPS[i];
      setUpdProgress(pct);
      setUpdStatus(msg);
      if (pct === 100) {
        clearInterval(iv);
        setUpdChecking(false);
      }
      i += 1;
    }, 650);
  };

  const done = updProgress === 100;

  return (
    <div className="settings-screen">
      <div className="settings-layout rise">
        <nav className="stabs">
          {TABS.map((t) => (
            <button key={t.key} className={"stab" + (tab === t.key ? " on" : "")} onClick={() => setTab(t.key)}>
              {t.label}
            </button>
          ))}
        </nav>

        <div>
          {/* PROTECTION */}
          <div className={"panel" + (tab === "protection" ? " on" : "")}>
            <div className="card pad-lg">
              <div className="card-h"><h3>Protection engines</h3><span className="ch-sub">enable detection layers</span></div>
              <Opt title="Signature detection" sub="SHA-256/MD5 + rules · fastest, zero false-positives"><Toggle label="Signature detection" /></Opt>
              <Opt title="Heuristic detection" sub="Suspicious APIs, packers, script & PowerShell abuse"><Toggle label="Heuristic detection" /></Opt>
              <Opt title="Behavioral detection" sub="Runtime monitoring & ransomware behaviour"><Toggle label="Behavioral detection" /></Opt>
              <Opt title="ML risk scoring" sub="PE features + entropy for unknown files · may raise false-positives"><Toggle label="ML risk scoring" /></Opt>
            </div>
            <div className="card pad-lg mt-16">
              <div className="card-h"><h3>Sensitivity</h3></div>
              <Opt title="Detection threshold" sub="Higher = catches more, more false positives">
                <select className="sel-input" defaultValue="Balanced (recommended)">
                  <option>Balanced (recommended)</option>
                  <option>Aggressive</option>
                  <option>Cautious</option>
                </select>
              </Opt>
              <Opt title="Auto-quarantine malicious" sub="Isolate high-confidence threats without prompting"><Toggle label="Auto-quarantine malicious" /></Opt>
            </div>
          </div>

          {/* SCANNING */}
          <div className={"panel" + (tab === "scanning" ? " on" : "")}>
            <div className="card pad-lg">
              <div className="card-h"><h3>Scan behaviour</h3></div>
              <Opt title="Scan inside archives" sub="ZIP, RAR, 7z, TAR, GZIP — recursive"><Toggle label="Scan inside archives" /></Opt>
              <Opt title="Scan hidden & system files" sub="Include protected OS directories"><Toggle label="Scan hidden and system files" /></Opt>
              <Opt title="Incremental scanning" sub="Skip unchanged files using the cache"><Toggle label="Incremental scanning" /></Opt>
              <Opt title="Worker threads" sub="More threads = faster, higher CPU">
                <select className="sel-input" defaultValue="Auto (8)"><option>Auto (8)</option><option>4</option><option>16</option></select>
              </Opt>
              <Opt title="Max archive depth" sub="Guards against zip bombs">
                <select className="sel-input" defaultValue="5 levels"><option>5 levels</option><option>3 levels</option><option>10 levels</option></select>
              </Opt>
            </div>
            <div className="card pad-lg mt-16">
              <div className="card-h"><h3>Scheduled scans</h3></div>
              <Opt title="Daily quick scan" sub="02:00 · memory + persistence"><Toggle label="Daily quick scan" /></Opt>
              <Opt title="Weekly full scan" sub="Sunday 03:00 · all drives"><Toggle label="Weekly full scan" /></Opt>
            </div>
          </div>

          {/* UPDATES */}
          <div className={"panel" + (tab === "updates" ? " on" : "")}>
            <div className="update-card">
              <div className="upd-row">
                <div className="ico">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7"><path d="M21 12a9 9 0 1 1-3-6.7L21 8M21 3v5h-5" /></svg>
                </div>
                <div style={{ flex: 1 }}>
                  <div style={{ fontWeight: 600, fontSize: 14 }}>Signatures up to date</div>
                  <div className="muted" style={{ fontSize: 12.5 }}>v2024.06.22.02 · 1,420,338 signatures · checked 2h ago</div>
                </div>
                <button className="btn primary" onClick={runUpdate} disabled={updChecking}>
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" style={{ width: 15, height: 15 }}><path d="M21 12a9 9 0 1 1-3-6.7L21 8" /></svg>
                  Check now
                </button>
              </div>
              {updProgress !== null ? (
                <div className="bar-track mt-16">
                  <div className="bar-fill" style={{ width: updProgress + "%", background: done ? "var(--accent)" : undefined }} />
                </div>
              ) : null}
              {updStatus ? (
                <div className="mt-8" style={{ fontSize: 12, color: done ? "var(--accent)" : "var(--muted)" }}>{updStatus}</div>
              ) : null}
              <ul className="secure-list">
                <li><CheckGlyph />Delta updates downloaded over TLS 1.3</li>
                <li><CheckGlyph />Verified against a pinned signing key before install</li>
                <li><CheckGlyph />Atomic swap with automatic rollback on failure</li>
              </ul>
            </div>
            <div className="card pad-lg mt-16">
              <div className="card-h"><h3>Update channel</h3></div>
              <Opt title="Automatic updates" sub="Check every 2 hours"><Toggle label="Automatic updates" /></Opt>
              <Opt title="Release channel" sub="Stable signature feed">
                <select className="sel-input" defaultValue="Stable"><option>Stable</option><option>Beta (faster, riskier)</option></select>
              </Opt>
            </div>
          </div>

          {/* EXCLUSIONS */}
          <div className={"panel" + (tab === "exclusions" ? " on" : "")}>
            <div className="card pad-lg">
              <div className="card-h"><h3>Excluded paths &amp; files</h3><span className="ch-sub">never scanned — use with care</span>
                <div className="ch-right"><button className="btn ghost sm">+ Add exclusion</button></div>
              </div>
              {["C:\\Dev\\node_modules", "*.iso", "D:\\VMs"].map((p) => (
                <div className="excl-row" key={p}>
                  <svg viewBox="0 0 24 24" fill="none" stroke="var(--muted)" strokeWidth="1.7" style={{ width: 15, height: 15 }}><path d="M3 7h7l2 2h9v10H3z" /></svg>
                  <span className="fp">{p}</span>
                  <button className="icon-btn" style={{ marginLeft: "auto", width: 28, height: 28 }} aria-label={`Remove ${p}`}>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" style={{ width: 14, height: 14 }}><line x1="6" y1="6" x2="18" y2="18" /><line x1="18" y1="6" x2="6" y2="18" /></svg>
                  </button>
                </div>
              ))}
            </div>
          </div>

          {/* SECURITY */}
          <div className={"panel" + (tab === "security" ? " on" : "")}>
            <div className="card pad-lg">
              <div className="card-h"><h3>Tamper protection</h3></div>
              <Opt title="Protect Aegis files & service" sub="Block other apps from disabling or deleting Aegis"><Toggle label="Protect Aegis files and service" /></Opt>
              <Opt title="Require password to change settings" sub="Prevents malware from weakening protection"><Toggle defaultOn={false} label="Require password to change settings" /></Opt>
              <Opt title="Reject unsigned IPC clients" sub="Only the signed Aegis UI may talk to the engine"><Toggle label="Reject unsigned IPC clients" /></Opt>
            </div>
            <div className="card pad-lg mt-16">
              <div className="card-h"><h3>Privacy</h3></div>
              <Opt title="Local-only mode" sub="No cloud lookups; all analysis on-device"><Toggle label="Local-only mode" /></Opt>
              <Opt title="Share anonymous detection stats" sub="Help improve the open-source signature feed"><Toggle defaultOn={false} label="Share anonymous detection stats" /></Opt>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function CheckGlyph() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8">
      <path d="M9 11l3 3L22 4" />
    </svg>
  );
}
