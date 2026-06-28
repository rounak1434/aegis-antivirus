import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Icon } from "../../components/Icon";
import { ErrorBanner } from "../../components/States";
import { useHealthStore } from "../../stores/healthStore";
import { useThreatStore } from "../../stores/threatStore";
import { useQuarantineStore } from "../../stores/quarantineStore";
import { useRealtimeStore } from "../../stores/realtimeStore";
import { useUpdateStore } from "../../stores/updateStore";
import type { ScanMode } from "../../types/ipc";
import { SCAN_MODES, TONE_VAR, levelTone } from "./data";
import "./dashboard.css";

const STATUS_TONE: Record<string, "ok" | "warn" | "danger"> = { ok: "ok", degraded: "warn", unavailable: "danger" };

/** Dashboard — live data from AegisService (health, threats, quarantine, RTP, updates). */
export function Dashboard() {
  const navigate = useNavigate();
  const [selectedMode, setSelectedMode] = useState<ScanMode>("deep");

  const { health, error: healthError, load: loadHealth } = useHealthStore();
  const { items: threats, load: loadThreats } = useThreatStore();
  const { items: quarantined, load: loadQuarantine } = useQuarantineStore();
  const { status: rt, load: loadRt } = useRealtimeStore();
  const { installed, load: loadUpdates } = useUpdateStore();

  useEffect(() => {
    void loadHealth();
    void loadThreats();
    void loadQuarantine();
    void loadRt();
    void loadUpdates();
  }, [loadHealth, loadThreats, loadQuarantine, loadRt, loadUpdates]);

  const protectedNow = health?.overall === "ok" && rt?.running;
  const sigVersion = installed.find(([c]) => c === "signature_database")?.[1] ?? "—";
  const topThreats = [...threats].sort((a, b) => b.score - a.score).slice(0, 6);

  return (
    <div className="dashboard-screen">
      {healthError ? <div className="mb-16"><ErrorBanner error={healthError} onRetry={() => void loadHealth()} /></div> : null}

      <div className="hero rise mb-16">
        <div className="posture">
          <div className="ring">
            <svg width="150" height="150" viewBox="0 0 150 150">
              <circle cx="75" cy="75" r="64" fill="none" stroke="rgba(255,255,255,0.07)" strokeWidth="11" />
              <circle cx="75" cy="75" r="64" fill="none" stroke="var(--accent)" strokeWidth="11" strokeLinecap="round" strokeDasharray="402" strokeDashoffset={protectedNow ? 36 : 200} />
            </svg>
            <div className="center"><Icon name="shieldCheck" size={46} strokeWidth={1.6} /></div>
          </div>
          <h2>{protectedNow ? "You're protected" : "Needs attention"}</h2>
          <p className="ps">
            {health ? `Service ${health.overall} · ${threats.length} active threat${threats.length === 1 ? "" : "s"}` : "Reading service status…"}
          </p>
          <div className="flex gap-8 mt-16">
            <span className={"pill " + (rt?.running ? "ok" : "muted")}><span className="dot" />{rt?.running ? "Real-time on" : "Real-time off"}</span>
            <span className="pill info">{sigVersion === "—" ? "No signatures" : "Defs " + sigVersion}</span>
          </div>
        </div>

        <div className="scan-launch">
          <div className="card-h">
            <h3>Run a scan</h3>
            <span className="ch-sub">{selectedMode[0].toUpperCase() + selectedMode.slice(1)} scan selected</span>
            <div className="ch-right">
              <button className="btn primary" onClick={() => navigate("/scan", { state: { mode: selectedMode } })}>
                <Icon name="scan" strokeWidth={1.8} />Start scan
              </button>
            </div>
          </div>
          <div className="modes">
            {SCAN_MODES.map((m) => (
              <button key={m.mode} className={"mode" + (selectedMode === m.mode ? " sel" : "")} onClick={() => setSelectedMode(m.mode)} aria-pressed={selectedMode === m.mode}>
                <Icon name={m.icon} strokeWidth={1.7} />
                <div className="mn">{m.label}</div>
                <div className="mm">{m.desc}</div>
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="grid g-4 mb-16 rise-2">
        <Stat icon="shield" label="Active threats" value={String(threats.length)} tone={threats.length ? "danger" : undefined} delta="from latest scans" />
        <Stat icon="box" label="In quarantine" value={String(quarantined.length)} tone={quarantined.length ? "warn" : undefined} delta="review vault →" onClick={() => navigate("/quarantine")} />
        <Stat icon="realtime" label="Real-time" value={rt?.running ? "on" : "off"} delta={rt ? `${rt.events_processed} events · ${rt.alerts_raised} alerts` : "—"} small />
        <Stat icon="clock" label="Signatures" value={sigVersion} small delta={installed.length ? `${installed.length} components` : "none installed"} />
      </div>

      <div className="grid g-2">
        <div className="card pad-lg rise-2">
          <div className="card-h"><h3>Service health</h3></div>
          {health ? (
            <>
              <HealthRow label="Scanner" status={health.scanner} />
              <HealthRow label="Database" status={health.database} />
              <HealthRow label="YARA rules" status={health.rules} />
              <HealthRow label="Quarantine vault" status={health.quarantine} />
              <HealthRow label="Active jobs" status="ok" value={String(health.active_jobs)} />
            </>
          ) : (
            <div className="muted" style={{ fontSize: 13 }}>Reading service health…</div>
          )}
        </div>

        <div className="card pad-lg rise-3">
          <div className="card-h">
            <h3>Latest threats</h3>
            <span className="ch-right"><a href="/threats" className="text-info" style={{ fontSize: 12 }} onClick={(e) => { e.preventDefault(); navigate("/threats"); }}>all threats →</a></span>
          </div>
          {topThreats.length === 0 ? (
            <div className="muted" style={{ fontSize: 13 }}>No threats detected.</div>
          ) : (
            topThreats.map((t) => (
              <div className="act-item" key={t.id}>
                <span className="ad" style={{ background: TONE_VAR[levelTone(t.threat_level)] }} />
                <div>
                  <div className="at"><b>{t.threat_level}</b> {fileName(t.path)}</div>
                  <div className="as">{t.path} · score {t.score}</div>
                </div>
                <time>{t.threat_level}</time>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

function fileName(p: string): string {
  return p.split(/[\\/]/).pop() ?? p;
}

function Stat({ icon, label, value, tone, delta, small, onClick }: { icon: Parameters<typeof Icon>[0]["name"]; label: string; value: string; tone?: "danger" | "warn"; delta: string; small?: boolean; onClick?: () => void }) {
  return (
    <div className="card stat">
      <div className="s-label"><Icon name={icon} strokeWidth={1.7} />{label}</div>
      <div className={"s-val" + (tone ? " text-" + tone : "")} style={small ? { fontSize: 22 } : undefined}>{value}</div>
      <div className="s-delta">{onClick ? <a href="#" className="text-info" onClick={(e) => { e.preventDefault(); onClick(); }}>{delta}</a> : delta}</div>
    </div>
  );
}

function HealthRow({ label, status, value }: { label: string; status: string; value?: string }) {
  return (
    <div className="surface-row">
      <div><div className="sn">{label}</div></div>
      <div className="sv">
        {value !== undefined ? <span className="mono" style={{ fontSize: 13 }}>{value}</span> : (
          <span className={"pill " + STATUS_TONE[status]}><span className="dot" />{status}</span>
        )}
      </div>
    </div>
  );
}
