import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Icon } from "../../components/Icon";
import type { ScanMode } from "../../types/ipc";
import {
  SCAN_MODES,
  DASH_STATS,
  PERSISTENCE_SURFACES,
  RECENT_ACTIVITY,
  TONE_VAR
} from "./data";
import "./dashboard.css";

/**
 * Dashboard screen — faithful React conversion of
 * design-prototype/dashboard.html. Mock data lives in ./data.ts (Phase C will
 * swap it for the Zustand store; Phase D for live IPC). Visual structure,
 * classes, and copy are preserved 1:1 with the prototype.
 */
export function Dashboard() {
  const navigate = useNavigate();
  const [selectedMode, setSelectedMode] = useState<ScanMode>("deep");

  return (
    <div className="dashboard-screen">
      <div className="hero rise mb-16">
        <div className="posture">
          <div className="ring">
            <svg width="150" height="150" viewBox="0 0 150 150">
              <circle cx="75" cy="75" r="64" fill="none" stroke="rgba(255,255,255,0.07)" strokeWidth="11" />
              <circle
                cx="75"
                cy="75"
                r="64"
                fill="none"
                stroke="var(--accent)"
                strokeWidth="11"
                strokeLinecap="round"
                strokeDasharray="402"
                strokeDashoffset="36"
              />
            </svg>
            <div className="center">
              <Icon name="shieldCheck" size={46} strokeWidth={1.6} />
            </div>
          </div>
          <h2>You&apos;re protected</h2>
          <p className="ps">All shields active · 1 item needs review</p>
          <div className="flex gap-8 mt-16">
            <span className="pill ok"><span className="dot" />Real-time on</span>
            <span className="pill info">Defs current</span>
          </div>
        </div>

        <div className="scan-launch">
          <div className="card-h">
            <h3>Run a scan</h3>
            <span className="ch-sub">{labelForMode(selectedMode)} scan selected · last full scan 2 days ago</span>
            <div className="ch-right">
              <button className="btn primary" onClick={() => navigate("/scan")}>
                <Icon name="scan" strokeWidth={1.8} />
                Start scan
              </button>
            </div>
          </div>
          <div className="modes">
            {SCAN_MODES.map((m) => (
              <button
                key={m.mode}
                className={"mode" + (selectedMode === m.mode ? " sel" : "")}
                onClick={() => setSelectedMode(m.mode)}
                aria-pressed={selectedMode === m.mode}
              >
                <Icon name={m.icon} strokeWidth={1.7} />
                <div className="mn">{m.label}</div>
                <div className="mm">{m.desc}</div>
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="grid g-4 mb-16 rise-2">
        {DASH_STATS.map((s) => (
          <div className="card stat" key={s.label}>
            <div className="s-label">
              <Icon name={s.icon} strokeWidth={1.7} />
              {s.label}
            </div>
            <div
              className={"s-val" + (s.valueTone ? " text-" + s.valueTone : "")}
              style={s.valueSmall ? { fontSize: 22 } : undefined}
            >
              {s.value}
            </div>
            <div className={"s-delta" + (s.deltaTone ? " " + s.deltaTone : "")}>
              {s.deltaLink ? (
                <a href={s.deltaLink.href} className="text-info" onClick={linkNav(navigate, s.deltaLink.href)}>
                  {s.deltaLink.text}
                </a>
              ) : (
                s.delta
              )}
            </div>
          </div>
        ))}
      </div>

      <div className="grid g-2">
        <div className="card pad-lg rise-2">
          <div className="card-h">
            <h3>Persistence surface health</h3>
            <span className="ch-right">
              <a href="/threats" className="text-info" style={{ fontSize: 12 }} onClick={linkNav(navigate, "/threats")}>
                1 finding →
              </a>
            </span>
          </div>
          {PERSISTENCE_SURFACES.map((row) => (
            <div className="surface-row" key={row.name}>
              <div className="si">
                <Icon name={row.icon} strokeWidth={1.7} />
              </div>
              <div>
                <div className="sn">{row.name}</div>
                <div className="sd">{row.detail}</div>
              </div>
              <div className="sv">
                <span className={"pill " + row.status}>
                  <span className="dot" />
                  {row.statusLabel}
                </span>
              </div>
            </div>
          ))}
        </div>

        <div className="card pad-lg rise-3">
          <div className="card-h">
            <h3>Recent activity</h3>
            <span className="ch-right">
              <a href="/realtime" className="text-info" style={{ fontSize: 12 }} onClick={linkNav(navigate, "/realtime")}>
                live feed →
              </a>
            </span>
          </div>
          {RECENT_ACTIVITY.map((a, i) => (
            <div className="act-item" key={i}>
              <span className="ad" style={{ background: TONE_VAR[a.tone] }} />
              <div>
                <div className="at">{boldLeadVerb(a.title)}</div>
                <div className="as">{a.detail}</div>
              </div>
              <time>{a.time}</time>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function labelForMode(mode: ScanMode): string {
  return mode.charAt(0).toUpperCase() + mode.slice(1);
}

/** Intercept internal anchor clicks so they route through react-router. */
function linkNav(navigate: ReturnType<typeof useNavigate>, to: string) {
  return (e: React.MouseEvent) => {
    e.preventDefault();
    navigate(to);
  };
}

/** Bold the leading verb, matching the prototype's `<b>Blocked</b> …` markup. */
function boldLeadVerb(title: string) {
  const space = title.indexOf(" ");
  if (space === -1) return title;
  return (
    <>
      <b>{title.slice(0, space)}</b>
      {title.slice(space)}
    </>
  );
}
