import type { IconName } from "../../components/Icon";
import type { ScanMode } from "../../types/ipc";

/** Scan-mode presentation config (not mock data — these define the UI choices). */
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

export type ActivityTone = "danger" | "warn" | "info" | "accent";

export const TONE_VAR: Record<ActivityTone, string> = {
  danger: "var(--danger)",
  warn: "var(--warn)",
  info: "var(--info)",
  accent: "var(--accent)"
};

/** Map a threat level to a tone for the activity dots. */
export function levelTone(level: string): ActivityTone {
  if (level === "critical" || level === "high") return "danger";
  if (level === "medium") return "warn";
  if (level === "low") return "info";
  return "accent";
}
