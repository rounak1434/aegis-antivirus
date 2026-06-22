import { AppShell } from "./components/AppShell";
import { Dashboard } from "./features/dashboard/Dashboard";

export function App() {
  return (
    <AppShell>
      <Dashboard />
    </AppShell>
  );
}
