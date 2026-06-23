import { useNavigate } from "react-router-dom";
import { Icon } from "../../components/Icon";
import { useScanSimulation } from "./useScanSimulation";
import type { ScanDetection } from "./useScanSimulation";
import "./scan.css";

const RING_CIRC = 540;

function sevColor(n: number) {
  return n >= 85 ? "var(--danger)" : n >= 60 ? "var(--warn)" : "var(--info)";
}

/**
 * Scan Center — React conversion of design-prototype/scan.html. Live progress
 * comes from useScanSimulation (swapped for IPC ScanProgress events in Phase D).
 */
export function ScanCenter() {
  const navigate = useNavigate();
  const { state, togglePause } = useScanSimulation();
  const { engines } = state;

  return (
    <div className="scan-screen">
      <div className="card pad-lg rise mb-16">
        <div className="scan-hero">
          <div className="scan-ring">
            <svg width="200" height="200" viewBox="0 0 200 200">
              <circle cx="100" cy="100" r="86" fill="none" stroke="rgba(255,255,255,0.06)" strokeWidth="13" />
              <circle
                cx="100"
                cy="100"
                r="86"
                fill="none"
                stroke="var(--accent)"
                strokeWidth="13"
                strokeLinecap="round"
                strokeDasharray={RING_CIRC}
                strokeDashoffset={RING_CIRC * (1 - state.pct / 100)}
                style={{ transition: "stroke-dashoffset .38s linear" }}
              />
            </svg>
            <div className="c">
              <div className="pct">{Math.floor(state.pct)}%</div>
              <div className="pl">{state.finished ? "complete" : state.paused ? "paused" : "scanning"}</div>
            </div>
          </div>
          <div className="scan-meta">
            <div className="flex between items-center">
              <div>
                <h2>{state.finished ? "Deep scan complete" : "Deep scan"}</h2>
                <div className="sub">All drives · archives · memory · all detection layers</div>
              </div>
              <div className="flex gap-8">
                {state.finished ? (
                  <button className="btn primary sm" onClick={() => navigate("/threats")}>
                    View report →
                  </button>
                ) : (
                  <>
                    <button className="btn ghost sm" onClick={togglePause}>
                      {state.paused ? (
                        <Icon name="bolt" strokeWidth={1.8} />
                      ) : (
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" style={{ width: 15, height: 15 }}>
                          <rect x="6" y="5" width="4" height="14" />
                          <rect x="14" y="5" width="4" height="14" />
                        </svg>
                      )}
                      {state.paused ? "Resume" : "Pause"}
                    </button>
                    <button className="btn danger sm" onClick={() => navigate("/threats")}>
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" style={{ width: 15, height: 15 }}>
                        <line x1="6" y1="6" x2="18" y2="18" />
                        <line x1="18" y1="6" x2="6" y2="18" />
                      </svg>
                      Stop
                    </button>
                  </>
                )}
              </div>
            </div>
            <div className="cur-path">
              <span className="tag">{state.finished ? "✓ done ›" : "scanning ›"}</span> {state.curPath}
            </div>
            <div className="engines">
              <Engine label="File walker" count={engines.walk} />
              <Engine label="Signature match" count={engines.sig} />
              <Engine label="Heuristic + PE" count={engines.heur} />
              <Engine label="Archive extract" count={engines.arch} idle={!state.archActive} />
            </div>
          </div>
        </div>
      </div>

      <div className="grid g-4 mb-16 rise-2">
        <Stat label="Scanned" value={state.scanned} delta="of ~241,000 est." />
        <Stat label="Throughput" value={state.throughput} unit="f/s" delta="8 worker threads" small />
        <Stat label="Detections" value={String(state.detections.length)} tone="danger" delta="live as found" small />
        <Stat label="Elapsed" value={state.elapsed} delta={state.eta} small mono />
      </div>

      <div className="card pad-lg rise-3">
        <div className="card-h">
          <h3>Live detections</h3>
          <span className="ch-sub">streamed from the verdict aggregator</span>
          <span className="ch-right">
            <span className="pill danger">
              <span className="dot" />
              {state.detections.length} found
            </span>
          </span>
        </div>
        <div className="feed">
          {state.detections.length === 0 ? (
            <div className="feed-row" style={{ color: "var(--muted)" }}>
              <span />
              <span className="fp">No detections yet — scanning clean files…</span>
              <span />
            </div>
          ) : (
            state.detections.map((d, i) => <DetectionRow key={d.path + i} det={d} />)
          )}
        </div>
      </div>
    </div>
  );
}

function Engine({ label, count, idle }: { label: string; count: string; idle?: boolean }) {
  return (
    <div className={"eng" + (idle ? " idle" : "")}>
      <span className="pulse" />
      <span className="en">{label}</span>
      <span className="ec">{count}</span>
    </div>
  );
}

function Stat({ label, value, unit, delta, tone, small, mono }: { label: string; value: string; unit?: string; delta: string; tone?: "danger"; small?: boolean; mono?: boolean }) {
  return (
    <div className="card stat">
      <div className="s-label">{label}</div>
      <div className={"s-val" + (tone ? " text-" + tone : "") + (mono ? " mono" : "")} style={small ? { fontSize: 22 } : undefined}>
        {value}
        {unit ? <span className="u">{unit}</span> : null}
      </div>
      <div className="s-delta">{delta}</div>
    </div>
  );
}

function DetectionRow({ det }: { det: ScanDetection }) {
  return (
    <div className="feed-row rise">
      <span className="pill danger" style={{ fontSize: 10 }}>
        <span className="dot" />
        {det.name.split(":")[0]}
      </span>
      <span className="col">
        <span className="fp">{det.path}</span>
        <span className="sub" style={{ fontSize: 11, color: "var(--muted)" }}>
          {det.name} · {det.reason}
        </span>
      </span>
      <span className="sev">
        <span className="bar">
          <i style={{ width: det.severity + "%", background: sevColor(det.severity) }} />
        </span>
        {det.severity}
      </span>
    </div>
  );
}
