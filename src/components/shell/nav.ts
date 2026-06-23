import type { IconName } from "../Icon";

/**
 * Sidebar navigation model, ported from design-prototype/js/shell.js `NAV`.
 * Each item carries the React route plus the per-screen metadata that the
 * prototype encoded in `<body data-active data-title data-crumb data-wide>`.
 */
export interface NavItem {
  key: string;
  path: string;
  label: string;
  icon: IconName;
  title: string;
  crumb: string;
  wide?: boolean;
  badge?: string;
  /** Screens converted so far. Others render the interim section view. */
  ready: boolean;
}

export interface NavGroup {
  group: string;
  items: NavItem[];
}

export const NAV: NavGroup[] = [
  {
    group: "Overview",
    items: [
      { key: "launcher", path: "/launcher", label: "Launcher", icon: "home", title: "Launcher", crumb: "Aegis / overview", ready: false },
      { key: "dashboard", path: "/", label: "Dashboard", icon: "dashboard", title: "Dashboard", crumb: "Aegis / overview", ready: true },
      { key: "arch", path: "/architecture", label: "Architecture", icon: "arch", title: "Architecture & system design", crumb: "Aegis / engineering", wide: true, ready: false }
    ]
  },
  {
    group: "Protect",
    items: [
      { key: "scan", path: "/scan", label: "Scan", icon: "scan", title: "Deep scan in progress", crumb: "Aegis / scan", ready: false },
      { key: "threats", path: "/threats", label: "Threats", icon: "threats", title: "Threats & report", crumb: "Aegis / detections", badge: "3", ready: false },
      { key: "quarantine", path: "/quarantine", label: "Quarantine", icon: "quarantine", title: "Quarantine vault", crumb: "Aegis / quarantine", ready: false },
      { key: "realtime", path: "/realtime", label: "Real-time", icon: "realtime", title: "Real-time protection", crumb: "Aegis / protection", ready: false }
    ]
  },
  {
    group: "System",
    items: [
      { key: "widget", path: "/widget", label: "Mini widget", icon: "widget", title: "Mini widget & system tray", crumb: "Aegis / system", ready: false },
      { key: "settings", path: "/settings", label: "Settings", icon: "settings", title: "Settings & updates", crumb: "Aegis / settings", ready: false }
    ]
  }
];

export const NAV_ITEMS: NavItem[] = NAV.flatMap((g) => g.items);

export function navItemByPath(pathname: string): NavItem | undefined {
  return NAV_ITEMS.find((item) => item.path === pathname);
}
