import { HashRouter, Routes, Route } from "react-router-dom";
import { AppShell } from "./components/AppShell";
import { SectionComingNext } from "./components/SectionComingNext";
import { Dashboard } from "./features/dashboard/Dashboard";
import { ScanCenter } from "./features/scan-center/ScanCenter";
import { ThreatCenter } from "./features/threat-center/ThreatCenter";
import { Quarantine } from "./features/quarantine/Quarantine";
import { Realtime } from "./features/realtime/Realtime";
import { Updates } from "./features/updates/Updates";
import { Settings } from "./features/settings/Settings";

/**
 * Application router. HashRouter is used because the app is served from a
 * file/tauri origin where path-based routing is unreliable. The AppShell is a
 * layout route (winbar + sidebar + topbar); screens render into its Outlet.
 *
 * Converted: Dashboard, Scan Center, Threat Center, Quarantine, Real-time,
 * Settings. Launcher and Architecture render the interim view until ported.
 */
export function App() {
  return (
    <HashRouter>
      <Routes>
        <Route element={<AppShell />}>
          <Route index element={<Dashboard />} />
          <Route path="scan" element={<ScanCenter />} />
          <Route path="threats" element={<ThreatCenter />} />
          <Route path="quarantine" element={<Quarantine />} />
          <Route path="realtime" element={<Realtime />} />
          <Route path="updates" element={<Updates />} />
          <Route path="settings" element={<Settings />} />
          <Route path="launcher" element={<SectionComingNext />} />
          <Route path="architecture" element={<SectionComingNext />} />
          <Route path="widget" element={<SectionComingNext />} />
          <Route path="*" element={<SectionComingNext />} />
        </Route>
      </Routes>
    </HashRouter>
  );
}
