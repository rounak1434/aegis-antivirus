import { useEffect, useRef, useState } from "react";

/**
 * Self-contained deep-scan progress simulation, faithfully reproducing the
 * behavior in design-prototype/scan.html's inline script. This is the UI's
 * temporary data source; in Phase D it is replaced by real ScanProgress events
 * streamed from AegisService over IPC. The component consuming it does not
 * change when that swap happens — only the hook's internals.
 */

export interface ScanDetection {
  family: string;
  name: string;
  path: string;
  reason: string;
  severity: number;
}

const PATHS = [
  "C:\\Windows\\System32\\drivers\\",
  "C:\\Program Files\\",
  "C:\\Users\\admin\\AppData\\Local\\Temp\\",
  "C:\\Users\\admin\\Downloads\\",
  "C:\\ProgramData\\",
  "C:\\Windows\\SysWOW64\\",
  "D:\\Backups\\archive.zip › ",
  "C:\\Users\\admin\\Documents\\",
  "C:\\Windows\\Tasks\\"
];

const FILES = [
  "ntoskrnl.exe", "svchost.dll", "setup.tmp", "invoice_2024.exe", "update.ps1",
  "config.dat", "report.pdf", "driver32.sys", "cache.bin", "launcher.scr", "readme.txt", "7za.dll"
];

const DETS: ScanDetection[] = [
  { path: "C:\\Users\\admin\\Downloads\\invoice_2024.exe", name: "Trojan:Win32/Wacatac.B!ml", reason: "ML score 0.96 · packed PE", family: "Trojan", severity: 92 },
  { path: "C:\\Users\\admin\\AppData\\Local\\Temp\\update.ps1", name: "Heuristic:PowerShell/Obfus", reason: "base64 + IEX download cradle", family: "Heuristic", severity: 74 },
  { path: "D:\\Backups\\archive.zip › setup.scr", name: "Trojan:Win32/Masquerade", reason: "double extension · icon spoof", family: "Trojan", severity: 81 }
];

const TOTAL = 241000;
const TICK_MS = 380;

function pad(n: number) { return (n < 10 ? "0" : "") + n; }
function fmt(n: number) { return n.toLocaleString("en-US"); }

export interface ScanState {
  pct: number;
  scanned: string;
  throughput: string;
  elapsed: string;
  eta: string;
  curPath: string;
  engines: { walk: string; sig: string; heur: string; arch: string };
  archActive: boolean;
  detections: ScanDetection[];
  paused: boolean;
  finished: boolean;
}

export function useScanSimulation() {
  const [state, setState] = useState<ScanState>({
    pct: 0,
    scanned: "0",
    throughput: "0",
    elapsed: "00:00",
    eta: "est. —",
    curPath: "C:\\Windows\\System32\\drivers\\…",
    engines: { walk: "—", sig: "—", heur: "—", arch: "queued" },
    archActive: false,
    detections: [],
    paused: false,
    finished: false
  });

  const ref = useRef({ scanned: 0, elapsedMs: 0, shown: 0, paused: false, finished: false });

  useEffect(() => {
    const r = ref.current;
    const id = setInterval(() => {
      if (r.paused || r.finished) return;
      const inc = Math.floor(900 + Math.random() * 1600);
      r.scanned = Math.min(TOTAL, r.scanned + inc);
      r.elapsedMs += TICK_MS;
      const pct = Math.min(100, (r.scanned / TOTAL) * 100);
      const el = r.elapsedMs / 1000;
      const etaSec = pct > 2 ? el / (pct / 100) - el : 0;
      const archActive = false;
      const newDets: ScanDetection[] = [];
      if (r.shown < DETS.length && pct > (r.shown + 1) * 24) {
        newDets.push(DETS[r.shown]);
        r.shown += 1;
      }
      const curPath = PATHS[Math.floor(Math.random() * PATHS.length)] + FILES[Math.floor(Math.random() * FILES.length)];
      const finished = pct >= 100;
      r.finished = finished;

      setState((prev) => ({
        pct,
        scanned: fmt(r.scanned),
        throughput: fmt(inc * 2),
        elapsed: pad(Math.floor(el / 60)) + ":" + pad(Math.floor(el % 60)),
        eta: pct < 100 ? "est. " + pad(Math.floor(etaSec / 60)) + ":" + pad(Math.floor(etaSec % 60)) + " left" : "done",
        curPath: finished ? `241,000 files scanned · ${r.shown} threats found` : curPath,
        engines: {
          walk: fmt(r.scanned),
          sig: fmt(Math.floor(r.scanned * 0.62)),
          heur: fmt(Math.floor(r.scanned * 0.21)),
          arch: curPath.indexOf(".zip") > -1 || prev.archActive ? fmt(Math.floor(r.scanned * 0.04)) : "queued"
        },
        archActive: curPath.indexOf(".zip") > -1 || prev.archActive || archActive,
        detections: newDets.length ? [...newDets, ...prev.detections] : prev.detections,
        paused: r.paused,
        finished
      }));
    }, TICK_MS);
    return () => clearInterval(id);
  }, []);

  const togglePause = () => {
    ref.current.paused = !ref.current.paused;
    setState((prev) => ({ ...prev, paused: ref.current.paused }));
  };

  return { state, togglePause };
}
