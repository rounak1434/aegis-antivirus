import { useEffect } from "react";
import { Icon } from "../../components/Icon";
import { ErrorBanner } from "../../components/States";
import { useRealtimeStore } from "../../stores/realtimeStore";
import type { ProtectionMode } from "../../types/ipc";
import "./realtime.css";

const MODES: { mode: ProtectionMode; label: string; desc: string }[] = [
  { mode: "monitor_only", label: "Monitor only", desc: "Observe and log; never act." },
  { mode: "notify_only", label: "Notify only", desc: "Alert on detections; take no action (default)." },
  { mode: "auto_quarantine", label: "Auto-quarantine", desc: "Quarantine high/critical; notify otherwise." }
];

/** Real-time protection — live engine state, policy, and monitoring from AegisService. */
export function Realtime() {
  const { status, error, load, start, stop } = useRealtimeStore();

  useEffect(() => { void load(); }, [load]);

  const running = status?.running ?? false;
  const mode = status?.mode ?? "notify_only";

  const setMode = (m: ProtectionMode) => void start(m); // (re)start under the chosen policy

  return (
    <div className="realtime-screen">
      {error ? <div className="mb-16"><ErrorBanner error={error} onRetry={() => void load()} /></div> : null}

      <div className="rt-banner rise">
        <Icon name="realtime" strokeWidth={1.7} />
        <div style={{ flex: 1 }}>
          <div style={{ fontWeight: 600, fontSize: 13.5 }}>Real-time protection is {running ? "active" : "off"}</div>
          <div style={{ fontSize: 12, color: "var(--muted)" }}>
            {status ? `${status.watched_paths.length} watched paths · ${status.events_processed} events · ${status.alerts_raised} alerts` : "Reading status…"}
          </div>
        </div>
        <span className={"pill " + (running ? "ok" : "muted")}><span className="dot" />{running ? "active" : "inactive"}</span>
      </div>

      <div className="grid g-2 mb-16 rise-2">
        <div className="card pad-lg">
          <div className="card-h"><h3>Protection engine</h3></div>
          <div className="shield">
            <div className="sh">
              <div className="ico"><Icon name="realtime" strokeWidth={1.7} /></div>
              <div><div className="st">File & process monitoring</div><div className="ss">notify (filesystem) + sysinfo (processes)</div></div>
              <button className={"toggle" + (running ? "" : " off")} onClick={() => (running ? void stop() : void start(mode))} role="switch" aria-checked={running} aria-label="Real-time protection" />
            </div>
            <div className="desc">Watches Downloads, Desktop, Documents, Temp, and the user profile; scans new/changed files and launched processes through the detection engine.</div>
          </div>
        </div>

        <div className="card pad-lg">
          <div className="card-h"><h3>Policy mode</h3><span className="ch-sub">applies on next detection</span></div>
          {MODES.map((m) => (
            <div className="opt" key={m.mode}>
              <div><div className="on-t">{m.label}</div><div className="on-s">{m.desc}</div></div>
              <button className={"toggle" + (mode === m.mode ? "" : " off")} onClick={() => setMode(m.mode)} role="radio" aria-checked={mode === m.mode} aria-label={m.label} />
            </div>
          ))}
        </div>
      </div>

      <div className="grid g-2">
        <div className="card pad-lg rise-2">
          <div className="card-h"><h3>Monitoring counters</h3><span className="ch-right"><span className="pill ok" style={{ fontSize: 10 }}><span className="dot" />{running ? "streaming" : "idle"}</span></span></div>
          <div className="grid g-2">
            <div className="card stat"><div className="s-label">Events processed</div><div className="s-val" style={{ fontSize: 22 }}>{status?.events_processed ?? 0}</div></div>
            <div className="card stat"><div className="s-label">Alerts raised</div><div className="s-val text-danger" style={{ fontSize: 22 }}>{status?.alerts_raised ?? 0}</div></div>
          </div>
        </div>

        <div className="card pad-lg rise-3">
          <div className="card-h"><h3>Watched folders</h3><span className="ch-sub">monitored for changes</span></div>
          {status && status.watched_paths.length > 0 ? (
            status.watched_paths.map((f) => (
              <div className="cf" key={f}>
                <svg viewBox="0 0 24 24" fill="none" stroke="var(--accent)" strokeWidth="1.7" style={{ width: 16, height: 16 }}><path d="M3 7h7l2 2h9v10H3z" /></svg>
                <span className="fp">{f}</span>
                <span className="pill ok" style={{ marginLeft: "auto", fontSize: 10 }}><span className="dot" />watched</span>
              </div>
            ))
          ) : (
            <div className="muted" style={{ fontSize: 13 }}>{running ? "No folders resolved on this host." : "Start protection to watch folders."}</div>
          )}
        </div>
      </div>
    </div>
  );
}
