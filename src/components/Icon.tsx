import type { CSSProperties } from "react";

/**
 * Inline-SVG icon set, ported verbatim from design-prototype/js/shell.js (the
 * `I` map + `ic()` helper). Stroke 1.7, 24x24, currentColor — identical to the
 * prototype so icons stay pixel-identical.
 */
export const ICON_PATHS = {
  dashboard: '<path d="M3 13h8V3H3zM13 21h8V8h-8zM13 3v3h8V3zM3 21h8v-5H3z"/>',
  scan: '<circle cx="11" cy="11" r="7"/><path d="M21 21l-4.3-4.3"/>',
  threats: '<path d="M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z"/><path d="M12 9v4M12 16h.01"/>',
  quarantine: '<rect x="3" y="4" width="18" height="4" rx="1"/><path d="M5 8v11a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V8"/><path d="M10 12h4"/>',
  realtime: '<path d="M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z"/><path d="M9 12l2 2 4-4"/>',
  history: '<path d="M3 3v5h5"/><path d="M3.05 13A9 9 0 1 0 6 5.3L3 8"/><path d="M12 7v5l3 2"/>',
  settings: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.6 1.6 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.6 1.6 0 0 0-2.7 1.1V21a2 2 0 0 1-4 0v-.1a1.6 1.6 0 0 0-2.7-1.1l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1A1.6 1.6 0 0 0 4 15a1.6 1.6 0 0 0-1.5-1H2a2 2 0 0 1 0-4h.1A1.6 1.6 0 0 0 4 8.6a1.6 1.6 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1A1.6 1.6 0 0 0 9 4V2a2 2 0 0 1 4 0v.1a1.6 1.6 0 0 0 2.7 1.1l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.6 1.6 0 0 0 1.1 2.7H22a2 2 0 0 1 0 4h-.1a1.6 1.6 0 0 0-1.5 1z"/>',
  arch: '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/>',
  widget: '<rect x="4" y="3" width="16" height="18" rx="2"/><path d="M9 8h6"/>',
  shield: '<path d="M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z"/>',
  shieldCheck: '<path d="M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z"/><path d="M9 12l2 2 4-4"/>',
  home: '<path d="M3 11l9-8 9 8"/><path d="M5 9v11h14V9"/>',
  bolt: '<path d="M13 2 3 14h7l-1 8 10-12h-7z"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="M21 21l-4.3-4.3"/>',
  bell: '<path d="M18 8a6 6 0 1 0-12 0c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.7 21a2 2 0 0 1-3.4 0"/>',
  list: '<path d="M3 7h18M3 12h18M3 17h12"/>',
  clock: '<circle cx="12" cy="12" r="9"/><path d="M12 8v4l3 2"/>',
  box: '<rect x="3" y="4" width="18" height="4" rx="1"/><path d="M5 8v12h14V8"/>',
  grid: '<rect x="3" y="3" width="18" height="18" rx="2"/>',
  globe: '<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a15 15 0 0 1 0 18"/>',
  wmi: '<path d="M3 7h18v10H3zM7 21h10"/>',
  registry: '<path d="M5 12h14M12 5v14"/>',
  folders: '<path d="M4 4h16v6H4zM4 14h16v6H4z"/>'
} as const;

export type IconName = keyof typeof ICON_PATHS;

interface IconProps {
  name: IconName;
  size?: number;
  strokeWidth?: number;
  className?: string;
  style?: CSSProperties;
}

export function Icon({ name, size, strokeWidth = 1.7, className, style }: IconProps) {
  const dimStyle: CSSProperties = size ? { width: size, height: size, ...style } : (style ?? {});
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      style={dimStyle}
      aria-hidden="true"
      focusable="false"
      dangerouslySetInnerHTML={{ __html: ICON_PATHS[name] }}
    />
  );
}
