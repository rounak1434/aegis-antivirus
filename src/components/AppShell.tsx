import type { ReactNode } from "react";

const navItems = ["Dashboard", "Scan Center", "Threat Center", "Quarantine", "Reports", "Settings"];

export function AppShell({ children }: { children: ReactNode }) {
  return (
    <div className="grid h-screen grid-cols-[260px_1fr] bg-surface text-fg">
      <aside className="border-r border-line bg-panel px-4 py-5">
        <div className="mb-8 flex items-center gap-3">
          <div className="grid h-10 w-10 place-items-center rounded-lg border border-accent/60 bg-accent/10 text-accent">A</div>
          <div>
            <div className="text-sm font-semibold tracking-[0.08em]">AEGIS</div>
            <div className="text-xs text-muted">AegisService controlled</div>
          </div>
        </div>
        <nav className="space-y-1">
          {navItems.map((item) => (
            <button key={item} className="w-full rounded-md px-3 py-2 text-left text-sm text-muted transition hover:bg-surface hover:text-fg">
              {item}
            </button>
          ))}
        </nav>
      </aside>
      <main className="flex min-w-0 flex-col">
        <header className="flex h-14 items-center justify-between border-b border-line px-6">
          <div>
            <div className="text-xs uppercase tracking-[0.08em] text-muted">Windows desktop antivirus</div>
            <h1 className="text-lg font-semibold tracking-[-0.01em]">Aegis Antivirus</h1>
          </div>
          <div className="rounded-full border border-accent/50 bg-accent/10 px-3 py-1 text-xs font-semibold text-accent">UI connected through IPC</div>
        </header>
        <section className="min-h-0 flex-1 overflow-auto p-6">{children}</section>
      </main>
    </div>
  );
}
