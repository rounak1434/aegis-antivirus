import { useEffect, useRef } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import { Icon } from "../../components/Icon";
import { ErrorBanner } from "../../components/States";
import { inTauri } from "../../lib/ipc";
import { useScanStore } from "../../stores/scanStore";
import type { ScanMode } from "../../types/ipc";
import "./scan.css";

const RING_CIRC = 540;

/** Default roots per scan mode (custom prompts for a folder). */
async function rootsForMode(mode: ScanMode): Promise<string[] | null> {
  if (mode === "custom") {
    if (!inTauri()) return ["C:\\"];
    const picked = await open({ directory: true, multiple: false, title: "Choose a folder to scan" });
    if (!picked || Array.isArray(picked)) return null;
    return [picked];
  }
  if (mode === "quick") return ["C:\\Users"];
  return ["C:\\"]; // full / deep
}

export function ScanCenter() {
  const navigate = useNavigate();
  const location = useLocation();
  const { activeJobId, job, starting, error, start, stop, poll } = useScanStore();
  const timer = useRef<ReturnType<typeof setInterval> | null>(null);

  const running = job?.status === "running" || job?.status === "queued";

  // Poll the active job while it is running.
  useEffect(() => {
    if (activeJobId && running) {
      timer.current = setInterval(() => void poll(), 500);
      return () => { if (timer.current) clearInterval(timer.current); };
    }
    if (timer.current) clearInterval(timer.current);
  }, [activeJobId, running, poll]);

  // Auto-start if navigated here with a mode (from the dashboard).
  const launched = useRef(false);
  useEffect(() => {
    const mode = (location.state as { mode?: ScanMode } | null)?.mode;
    if (mode && !launched.current) {
      launched.current = true;
      void launch(mode);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function launch(mode: ScanMode) {
    const roots = await rootsForMode(mode);
    if (!roots) return;
    await start(mode, roots);
  }

  const p = job?.progress;
  const pct = p?.percent ?? 0;
  const finished = job?.status === "completed" || job?.status === "cancelled" || job?.status === "failed";
  const etaText = p && p.eta_ms > 0 ? `est. ${fmtMs(p.eta_ms)} left` : finished ? "done" : "—";

  return (
    <div className="scan-screen">
      {error ? <div className="mb-16"><ErrorBanner error={error} /></div> : null}

      <div className="card pad-lg rise mb-16">
        <div className="scan-hero">
          <div className="scan-ring">
            <svg width="200" height="200" viewBox="0 0 200 200">
              <circle cx="100" cy="100" r="86" fill="none" stroke="rgba(255,255,255,0.06)" strokeWidth="13" />
              <circle cx="100" cy="100" r="86" fill="none" stroke="var(--accent)" strokeWidth="13" strokeLinecap="round"
                strokeDasharray={RING_CIRC} strokeDashoffset={RING_CIRC * (1 - pct / 100)} style={{ transition: "stroke-dashoffset .38s linear" }} />
            </svg>
            <div className="c">
              <div className="pct">{Math.floor(pct)}%</div>
              <div className="pl">{job?.status ?? "idle"}</div>
            </div>
          </div>
          <div className="scan-meta">
            <div className="flex between items-center">
              <div>
                <h2>{job ? `${job.job_type === "file_scan" ? "File scan" : "Windows scan"}` : "Start a scan"}</h2>
                <div className="sub">{activeJobId ? `job ${activeJobId.slice(0, 8)}` : "Pick a scan mode below"}</div>
              </div>
              <div className="flex gap-8">
                {running ? (
                  <button className="btn danger sm" onClick={() => void stop()}>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" style={{ width: 15, height: 15 }}><line x1="6" y1="6" x2="18" y2="18" /><line x1="18" y1="6" x2="6" y2="18" /></svg>
                    Cancel
                  </button>
                ) : finished && job?.threats_found ? (
                  <button className="btn primary sm" onClick={() => navigate("/threats")}>View threats →</button>
                ) : null}
              </div>
            </div>
            <div className="cur-path">
              <span className="tag">{finished ? "✓ done ›" : "scanning ›"}</span> {p?.current_path ?? "—"}
            </div>
            <div className="modes" style={{ marginTop: 14 }}>
              {(["quick", "full", "deep", "custom"] as ScanMode[]).map((m) => (
                <button key={m} className="btn ghost sm" disabled={running || starting} onClick={() => void launch(m)} style={{ textTransform: "capitalize" }}>
                  <Icon name="scan" strokeWidth={1.7} />{m}
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>

      <div className="grid g-4 mb-16 rise-2">
        <Stat label="Scanned" value={fmt(p?.files_scanned ?? 0)} delta={`of ${fmt(p?.total_files ?? 0)}`} />
        <Stat label="Throughput" value={fmt(Math.round(p?.files_per_sec ?? 0))} unit="f/s" delta="rayon workers" small />
        <Stat label="Threats" value={String(job?.threats_found ?? 0)} tone="danger" delta="after scan completes" small />
        <Stat label="Elapsed" value={fmtMs(p?.elapsed_ms ?? 0)} delta={etaText} small mono />
      </div>

      <div className="card pad-lg rise-3">
        <div className="card-h">
          <h3>Scan status</h3>
          <span className="ch-sub">live from AegisService JobManager</span>
        </div>
        <div className="feed">
          {!job ? (
            <div className="feed-row" style={{ color: "var(--muted)" }}><span /><span className="fp">No active scan. Choose a mode to begin.</span><span /></div>
          ) : (
            <div className="feed-row"><span /><span className="fp">{job.status} · {fmt(job.files_scanned)} files · {job.threats_found} threats{job.error ? ` · ${job.error}` : ""}</span><span /></div>
          )}
        </div>
      </div>
    </div>
  );
}

function fmt(n: number) { return n.toLocaleString("en-US"); }
function fmtMs(ms: number) {
  const s = Math.floor(ms / 1000);
  return `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;
}

function Stat({ label, value, unit, delta, tone, small, mono }: { label: string; value: string; unit?: string; delta: string; tone?: "danger"; small?: boolean; mono?: boolean }) {
  return (
    <div className="card stat">
      <div className="s-label">{label}</div>
      <div className={"s-val" + (tone ? " text-" + tone : "") + (mono ? " mono" : "")} style={small ? { fontSize: 22 } : undefined}>
        {value}{unit ? <span className="u">{unit}</span> : null}
      </div>
      <div className="s-delta">{delta}</div>
    </div>
  );
}
