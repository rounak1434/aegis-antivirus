import type { IconName } from "../../components/Icon";
import type { ScanMode } from "../../types/ipc";

/**
 * Typed seed data mirroring the illustrative values in
 * design-prototype/dashboard.html. These constants are the visual spec for the
 * Dashboard; in Phase C they are replaced by the dashboard/security Zustand
 * store, and in Phase D by live data from the AegisService IPC bridge.
 */

export interface ScanModeTile {
  mode: ScanMode;
  label: string;
  desc: string;
  icon: IconName;
}

export const SCAN_MODES: ScanModeTile[] = [
  { mode: "quick", label: "Quick", desc: "Memory + startup + hot paths · ~3 min", icon: "bolt" },
  { mode: "full", label: "Full", desc: "All drives & persistence · ~40 min", icon: "scan" },
  { mode: "deep", label: "Deep", desc: "+ archives, memory, all layers · ~1.5 h", icon: "shield" },
  { mode: "custom", label: "Custom", desc: "Pick paths, engines & depth", icon: "plus" }
];

export interface DashStat {
  icon: IconName;
  label: string;
  value: string;
  valueTone?: "danger" | "warn";
  valueSmall?: boolean;
  delta: string;
  deltaTone?: "up" | "down";
  deltaLink?: { href: string; text: string };
}

export const DASH_STATS: DashStat[] = [
  { icon: "list", label: "Files scanned (24h)", value: "128,440", delta: "incremental cache hit 71%", deltaTone: "up" },
  { icon: "shield", label: "Threats blocked (24h)", value: "2", valueTone: "danger", delta: "last block 4h ago" },
  { icon: "box", label: "In quarantine", value: "3", valueTone: "warn", delta: "", deltaLink: { href: "/quarantine", text: "review vault →" } },
  { icon: "clock", label: "Signatures", value: "1.42M", valueSmall: true, delta: "updated 2h ago · auto", deltaTone: "up" }
];

export type SurfaceStatus = "ok" | "warn";

export interface PersistenceSurface {
  icon: IconName;
  name: string;
  detail: string;
  status: SurfaceStatus;
  statusLabel: string;
}

export const PERSISTENCE_SURFACES: PersistenceSurface[] = [
  { icon: "registry", name: "Run / RunOnce registry keys", detail: "42 entries audited", status: "ok", statusLabel: "clean" },
  { icon: "folders", name: "Startup folders", detail: "11 entries audited", status: "ok", statusLabel: "clean" },
  { icon: "clock", name: "Scheduled tasks", detail: "1 unsigned task flagged", status: "warn", statusLabel: "review" },
  { icon: "grid", name: "Services & drivers", detail: "214 services · 96 drivers", status: "ok", statusLabel: "clean" },
  { icon: "wmi", name: "WMI persistence", detail: "event consumers audited", status: "ok", statusLabel: "clean" },
  { icon: "globe", name: "Browser extensions", detail: "Chrome 8 · Edge 5 · Firefox 3", status: "ok", statusLabel: "clean" }
];

export type ActivityTone = "danger" | "warn" | "info" | "accent";

export interface ActivityItem {
  tone: ActivityTone;
  title: string;
  detail: string;
  time: string;
}

export const RECENT_ACTIVITY: ActivityItem[] = [
  { tone: "danger", title: "Blocked Trojan:Win32/Wacatac in download", detail: "invoice_2024.exe → quarantined automatically", time: "4h ago" },
  { tone: "warn", title: "Flagged unsigned scheduled task", detail: "\\Microsoft\\Windows\\UpdaterCheck — review recommended", time: "6h ago" },
  { tone: "info", title: "Updated signature database", detail: "+2,104 signatures · v2024.06.22.02", time: "2h ago" },
  { tone: "accent", title: "Completed quick scan", detail: "18,902 files · 0 threats · 2m 51s", time: "8h ago" },
  { tone: "danger", title: "Blocked process injection attempt", detail: "powershell.exe → explorer.exe (RTP behavioral)", time: "1d ago" },
  { tone: "accent", title: "Completed full scan", detail: "241,338 files · 1 threat · 38m 12s", time: "2d ago" }
];

export const TONE_VAR: Record<ActivityTone, string> = {
  danger: "var(--danger)",
  warn: "var(--warn)",
  info: "var(--info)",
  accent: "var(--accent)"
};
