import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ErrorBanner } from "../../components/States";
import { useSettingsStore } from "../../stores/settingsStore";
import "./settings.css";

type TabKey = "protection" | "scanning" | "updates" | "exclusions" | "security";

const TABS: { key: TabKey; label: string }[] = [
  { key: "protection", label: "Protection" },
  { key: "scanning", label: "Scanning" },
  { key: "updates", label: "Updates" },
  { key: "exclusions", label: "Exclusions" },
  { key: "security", label: "Security" }
];

/** Store-bound toggle: state lives in settingsStore and is persisted via AegisService. */
function Toggle({ k, label, def = true }: { k: string; label: string; def?: boolean }) {
  const values = useSettingsStore((s) => s.values);
  const setVal = useSettingsStore((s) => s.set);
  const on = (values[k] as boolean | undefined) ?? def;
  return <button className={"toggle" + (on ? "" : " off")} onClick={() => setVal(k, !on)} role="switch" aria-checked={on} aria-label={label} />;
}

function Select({ k, options, def }: { k: string; options: string[]; def: string }) {
  const values = useSettingsStore((s) => s.values);
  const setVal = useSettingsStore((s) => s.set);
  const value = (values[k] as string | undefined) ?? def;
  return (
    <select className="sel-input" value={value} onChange={(e) => setVal(k, e.target.value)} aria-label={k}>
      {options.map((o) => <option key={o}>{o}</option>)}
    </select>
  );
}

function Opt({ title, sub, children }: { title: string; sub: string; children: React.ReactNode }) {
  return (
    <div className="opt">
      <div><div className="on-t">{title}</div><div className="on-s">{sub}</div></div>
      {children}
    </div>
  );
}

/** Settings — every value loaded from and persisted via AegisService. */
export function Settings() {
  const navigate = useNavigate();
  const [tab, setTab] = useState<TabKey>("protection");
  const { loading, saving, dirty, error, load, save } = useSettingsStore();

  useEffect(() => { void load(); }, [load]);

  return (
    <div className="settings-screen">
      {error ? <div className="mb-16"><ErrorBanner error={error} onRetry={() => void load()} /></div> : null}

      <div className="card-h mb-16">
        <h3>Settings</h3>
        <span className="ch-sub">{loading ? "loading…" : dirty ? "unsaved changes" : "saved"}</span>
        <div className="ch-right">
          <button className="btn primary" onClick={() => void save()} disabled={!dirty || saving}>{saving ? "Saving…" : "Save changes"}</button>
        </div>
      </div>

      <div className="settings-layout rise">
        <nav className="stabs">
          {TABS.map((t) => (
            <button key={t.key} className={"stab" + (tab === t.key ? " on" : "")} onClick={() => setTab(t.key)}>{t.label}</button>
          ))}
        </nav>

        <div>
          <div className={"panel" + (tab === "protection" ? " on" : "")}>
            <div className="card pad-lg">
              <div className="card-h"><h3>Protection engines</h3><span className="ch-sub">enable detection layers</span></div>
              <Opt title="Signature detection" sub="SHA-256/MD5 + rules"><Toggle k="sig_detection" label="Signature detection" /></Opt>
              <Opt title="Heuristic detection" sub="Packers, script & PowerShell abuse"><Toggle k="heuristics" label="Heuristic detection" /></Opt>
              <Opt title="YARA rules" sub="Compiled rule matching"><Toggle k="yara" label="YARA rules" /></Opt>
              <Opt title="ML risk scoring" sub="PE features + entropy"><Toggle k="ml_scoring" label="ML risk scoring" /></Opt>
            </div>
            <div className="card pad-lg mt-16">
              <div className="card-h"><h3>Sensitivity</h3></div>
              <Opt title="Detection threshold" sub="Higher = catches more"><Select k="threshold" def="Balanced" options={["Balanced", "Aggressive", "Cautious"]} /></Opt>
              <Opt title="Auto-quarantine malicious" sub="Isolate high-confidence threats"><Toggle k="auto_quarantine" label="Auto-quarantine" /></Opt>
            </div>
          </div>

          <div className={"panel" + (tab === "scanning" ? " on" : "")}>
            <div className="card pad-lg">
              <div className="card-h"><h3>Scan behaviour</h3></div>
              <Opt title="Scan inside archives" sub="ZIP, RAR, 7z — recursive"><Toggle k="scan_archives" label="Scan archives" /></Opt>
              <Opt title="Scan hidden & system files" sub="Include protected OS dirs"><Toggle k="scan_hidden" label="Scan hidden files" /></Opt>
              <Opt title="Incremental scanning" sub="Skip unchanged files"><Toggle k="incremental" label="Incremental scanning" /></Opt>
              <Opt title="Worker threads" sub="More = faster, higher CPU"><Select k="threads" def="Auto" options={["Auto", "4", "8", "16"]} /></Opt>
            </div>
            <div className="card pad-lg mt-16">
              <div className="card-h"><h3>Scheduled scans</h3></div>
              <Opt title="Daily quick scan" sub="02:00 · memory + persistence"><Toggle k="sched_daily" label="Daily quick scan" /></Opt>
              <Opt title="Weekly full scan" sub="Sunday 03:00 · all drives"><Toggle k="sched_weekly" label="Weekly full scan" /></Opt>
            </div>
          </div>

          <div className={"panel" + (tab === "updates" ? " on" : "")}>
            <div className="card pad-lg">
              <div className="card-h"><h3>Updates</h3><span className="ch-sub">signature & rule delivery</span></div>
              <Opt title="Automatic updates" sub="Check periodically"><Toggle k="auto_update" label="Automatic updates" /></Opt>
              <Opt title="Update schedule" sub="How often to check"><Select k="update_schedule" def="Daily" options={["Manual", "Daily", "Weekly", "Startup"]} /></Opt>
              <Opt title="Release channel" sub="Stable or beta feed"><Select k="channel" def="Stable" options={["Stable", "Beta"]} /></Opt>
              <div className="opt"><div><div className="on-t">Update center</div><div className="on-s">Installed components, check, install, rollback</div></div>
                <button className="btn ghost sm" onClick={() => navigate("/updates")}>Open →</button></div>
            </div>
          </div>

          <div className={"panel" + (tab === "exclusions" ? " on" : "")}>
            <div className="card pad-lg">
              <div className="card-h"><h3>Excluded paths</h3><span className="ch-sub">stored in service settings</span></div>
              <Opt title="Exclusion list" sub="Comma-separated paths/globs never scanned">
                <input className="sel-input" placeholder="C:\\Dev\\node_modules, *.iso"
                  value={(useSettingsStore.getState().values.exclusions as string) ?? ""}
                  onChange={(e) => useSettingsStore.getState().set("exclusions", e.target.value)} aria-label="exclusions" />
              </Opt>
            </div>
          </div>

          <div className={"panel" + (tab === "security" ? " on" : "")}>
            <div className="card pad-lg">
              <div className="card-h"><h3>Tamper protection</h3></div>
              <Opt title="Protect Aegis files & service" sub="Block disabling/deleting Aegis"><Toggle k="tamper_protect" label="Tamper protection" /></Opt>
              <Opt title="Require password to change settings" sub="Prevents malware weakening protection"><Toggle k="settings_password" label="Settings password" def={false} /></Opt>
              <Opt title="Reject unsigned IPC clients" sub="Only the signed UI may talk to the engine"><Toggle k="signed_ipc" label="Reject unsigned IPC" /></Opt>
            </div>
            <div className="card pad-lg mt-16">
              <div className="card-h"><h3>Privacy</h3></div>
              <Opt title="Local-only mode" sub="No cloud lookups"><Toggle k="local_only" label="Local-only mode" /></Opt>
              <Opt title="Share anonymous detection stats" sub="Help improve the feed"><Toggle k="share_stats" label="Share stats" def={false} /></Opt>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
