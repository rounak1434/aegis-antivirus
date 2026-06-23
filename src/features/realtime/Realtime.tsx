import { useState } from "react";
import { Icon } from "../../components/Icon";
import type { IconName } from "../../components/Icon";
import "./realtime.css";

interface Shield {
  key: string;
  icon: IconName;
  title: string;
  subtitle: string;
  desc: string;
  on: boolean;
}

const INITIAL_SHIELDS: Shield[] = [
  { key: "fs", icon: "list", title: "File system shield", subtitle: "on-access scan · minifilter", desc: "Scans files on create & modify before they're written. Blocks known-bad writes inline.", on: true },
  { key: "proc", icon: "registry", title: "Process & injection shield", subtitle: "ETW · execution monitoring", desc: "Watches process launches and cross-process memory writes; blocks injection into trusted processes.", on: true },
  { key: "ransom", icon: "shield", title: "Ransomware shield", subtitle: "controlled folder access", desc: "Detects mass-encryption behaviour and blocks untrusted apps from modifying protected folders.", on: true },
  { key: "dl", icon: "bolt", title: "Download shield", subtitle: "browser & SmartScreen-style", desc: "Scans downloaded files as they land and checks reputation before the user can open them.", on: true },
  { key: "reg", icon: "registry", title: "Registry shield", subtitle: "persistence keys", desc: "Alerts on writes to Run keys, services, and other autostart locations from untrusted processes.", on: true },
  { key: "net", icon: "globe", title: "Network shield", subtitle: "C2 / beacon detection", desc: "Flags suspicious outbound connections and known command-and-control endpoints. Currently off.", on: false }
];

interface RtEvent {
  tone: string;
  title: string;
  verb: string;
  detail: string;
  time: string;
}

const EVENTS: RtEvent[] = [
  { tone: "var(--danger)", verb: "Blocked", title: "process injection", detail: "powershell.exe → explorer.exe", time: "just now" },
  { tone: "var(--accent)", verb: "Scanned", title: "download · clean", detail: "setup_legit.msi · signed", time: "2m" },
  { tone: "var(--warn)", verb: "Allowed (prompted)", title: "registry write", detail: "HKCU\\...\\Run ← Spotify.exe", time: "6m" },
  { tone: "var(--danger)", verb: "Blocked", title: "ransomware-like behaviour", detail: "unknown.exe · 312 files in 4s → Documents", time: "11m" },
  { tone: "var(--accent)", verb: "Scanned", title: "file write · clean", detail: "report.docx · MS Word", time: "14m" }
];

const FOLDERS = ["C:\\Users\\admin\\Documents", "C:\\Users\\admin\\Pictures", "D:\\Work"];
const ALLOWED_APPS = ["WINWORD.EXE", "EXCEL.EXE", "photoshop.exe", "Code.exe"];

/** Real-time protection — React conversion of design-prototype/realtime.html. */
export function Realtime() {
  const [shields, setShields] = useState<Shield[]>(INITIAL_SHIELDS);
  const onCount = shields.filter((s) => s.on).length;

  const toggle = (key: string) =>
    setShields((prev) => prev.map((s) => (s.key === key ? { ...s, on: !s.on } : s)));

  return (
    <div className="realtime-screen">
      <div className="rt-banner rise">
        <Icon name="realtime" strokeWidth={1.7} />
        <div style={{ flex: 1 }}>
          <div style={{ fontWeight: 600, fontSize: 13.5 }}>Real-time protection is active</div>
          <div style={{ fontSize: 12, color: "var(--muted)" }}>
            {onCount} of 6 shields on · monitoring file, process, registry, network &amp; download events live
          </div>
        </div>
        <span className="pill ok"><span className="dot" />active</span>
      </div>

      <div className="shield-grid mb-16 rise-2">
        {shields.map((s) => (
          <div className={"shield" + (s.on ? "" : " off")} key={s.key}>
            <div className="sh">
              <div className="ico">
                <Icon name={s.icon} strokeWidth={1.7} />
              </div>
              <div>
                <div className="st">{s.title}</div>
                <div className="ss">{s.subtitle}</div>
              </div>
              <button
                className={"toggle" + (s.on ? "" : " off")}
                onClick={() => toggle(s.key)}
                role="switch"
                aria-checked={s.on}
                aria-label={s.title}
              />
            </div>
            <div className="desc">{s.desc}</div>
          </div>
        ))}
      </div>

      <div className="grid g-2">
        <div className="card pad-lg rise-2">
          <div className="card-h">
            <h3>Live protection events</h3>
            <span className="ch-right">
              <span className="pill ok" style={{ fontSize: 10 }}>
                <span className="dot" />streaming
              </span>
            </span>
          </div>
          {EVENTS.map((e, i) => (
            <div className="ev-row" key={i}>
              <span className="ed" style={{ background: e.tone }} />
              <div>
                <div className="em"><b>{e.verb}</b> {e.title}</div>
                <div className="es">{e.detail}</div>
              </div>
              <time>{e.time}</time>
            </div>
          ))}
        </div>

        <div className="card pad-lg rise-3">
          <div className="card-h">
            <h3>Controlled folder access</h3>
            <span className="ch-sub">protected from untrusted apps</span>
            <div className="ch-right">
              <button className="btn ghost sm">+ Add folder</button>
            </div>
          </div>
          {FOLDERS.map((f) => (
            <div className="cf" key={f}>
              <svg viewBox="0 0 24 24" fill="none" stroke="var(--accent)" strokeWidth="1.7" style={{ width: 16, height: 16 }}>
                <path d="M3 7h7l2 2h9v10H3z" />
              </svg>
              <span className="fp">{f}</span>
              <span className="pill ok" style={{ marginLeft: "auto", fontSize: 10 }}>
                <span className="dot" />protected
              </span>
            </div>
          ))}
          <div className="divider" />
          <div className="card-h">
            <h3 style={{ fontSize: 13 }}>Allowed apps</h3>
          </div>
          <div className="flex gap-8" style={{ flexWrap: "wrap" }}>
            {ALLOWED_APPS.map((a) => (
              <span className="pill muted" key={a}>{a}</span>
            ))}
            <button className="chip" style={{ fontSize: 11, padding: "4px 10px", borderRadius: 20, background: "var(--surface)", border: "1px solid var(--border)", color: "var(--fg-2)", cursor: "pointer" }}>
              + allow app
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
