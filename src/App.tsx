import { HashRouter, Routes, Route } from "react-router-dom";
import { AppShell } from "./components/AppShell";
import { SectionComingNext } from "./components/SectionComingNext";
import { Dashboard } from "./features/dashboard/Dashboard";

/**
 * Application router. HashRouter is used because the app is served from a
 * file/tauri origin where path-based routing is unreliable. The AppShell is a
 * layout route (winbar + sidebar + topbar); screens render into its Outlet.
 *
 * Migration status: Dashboard is fully converted. The remaining routes render
 * the interim SectionComingNext view until each is ported (see PROTOTYPE_AUDIT.md).
 */
export function App() {
  return (
    <HashRouter>
      <Routes>
        <Route element={<AppShell />}>
          <Route index element={<Dashboard />} />
          <Route path="launcher" element={<SectionComingNext />} />
          <Route path="architecture" element={<SectionComingNext />} />
          <Route path="scan" element={<SectionComingNext />} />
          <Route path="threats" element={<SectionComingNext />} />
          <Route path="quarantine" element={<SectionComingNext />} />
          <Route path="realtime" element={<SectionComingNext />} />
          <Route path="widget" element={<SectionComingNext />} />
          <Route path="settings" element={<SectionComingNext />} />
          <Route path="*" element={<SectionComingNext />} />
        </Route>
      </Routes>
    </HashRouter>
  );
}
