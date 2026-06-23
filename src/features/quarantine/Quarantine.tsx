import { useState } from "react";

import "./quarantine.css";

interface QItem {
  file: string;
  origin: string;
  threat: string;
  severity: "danger" | "warn";
  when: string;
  size: string;
}

const ITEMS: QItem[] = [
  { file: "invoice_2024.exe", origin: "C:\\Users\\admin\\Downloads", threat: "Trojan:Win32/Wacatac.B!ml", severity: "danger", when: "4h ago", size: "2.4 MB" },
  { file: "setup.scr", origin: "D:\\Backups\\archive.zip", threat: "Trojan:Win32/Masquerade", severity: "danger", when: "4h ago", size: "880 KB" },
  { file: "cracktool.exe", origin: "C:\\Users\\admin\\Desktop", threat: "PUA:Win32/KeyGen", severity: "warn", when: "2d ago", size: "9.1 MB" }
];

/** Quarantine vault — React conversion of design-prototype/quarantine.html. */
export function Quarantine() {
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const [removed, setRemoved] = useState<Set<number>>(new Set());

  const toggle = (i: number) => {
    setSelected((prev) => {
      const next = new Set(prev);
      next.has(i) ? next.delete(i) : next.add(i);
      return next;
    });
  };

  const visibleIndexes = ITEMS.map((_, i) => i).filter((i) => !removed.has(i));
  const allChecked = visibleIndexes.length > 0 && visibleIndexes.every((i) => selected.has(i));

  const toggleAll = () => {
    setSelected(() => (allChecked ? new Set() : new Set(visibleIndexes)));
  };

  const deleteRow = (i: number) => {
    if (window.confirm("Permanently shred this file? This cannot be undone.")) {
      setRemoved((prev) => new Set(prev).add(i));
      setSelected((prev) => {
        const next = new Set(prev);
        next.delete(i);
        return next;
      });
    }
  };

  const selCount = [...selected].filter((i) => !removed.has(i)).length;

  return (
    <div className="quarantine-screen">
      <div className="vault-banner rise">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7">
          <rect x="3" y="11" width="18" height="10" rx="2" />
          <path d="M7 11V7a5 5 0 0 1 10 0v4" />
        </svg>
        <div style={{ flex: 1 }}>
          <div className="vt">Encrypted vault — 3 items isolated</div>
          <div className="vs">Files are AES-GCM encrypted at rest; originals neutralised. Quarantined files cannot execute.</div>
        </div>
        <span className="pill ok"><span className="dot" />vault sealed</span>
      </div>

      <div className="grid g-3 mb-16 rise-2">
        <div className="card stat">
          <div className="s-label">Items in vault</div>
          <div className="s-val">3</div>
          <div className="s-delta">12.4 MB encrypted</div>
        </div>
        <div className="card stat">
          <div className="s-label">Auto-delete after</div>
          <div className="s-val" style={{ fontSize: 22 }}>30 <span className="u">days</span></div>
          <div className="s-delta">configurable in settings</div>
        </div>
        <div className="card stat">
          <div className="s-label">Restored (all time)</div>
          <div className="s-val" style={{ fontSize: 22 }}>1</div>
          <div className="s-delta">false positive · whitelisted</div>
        </div>
      </div>

      <div className="card pad-lg rise-2">
        <div className="card-h">
          <h3>Quarantined items</h3>
          <span className="ch-sub">select items to act in bulk</span>
        </div>

        <div className={"sel-bar" + (selCount === 0 ? " hidden" : "")}>
          <input type="checkbox" className="check" checked={allChecked} onChange={toggleAll} aria-label="Select all" />
          <span className="mono" style={{ fontSize: 12 }}>{selCount} selected</span>
          <div style={{ marginLeft: "auto" }} className="flex gap-8">
            <button className="btn ghost sm">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" style={{ width: 15, height: 15 }}>
                <path d="M3 3v6h6M3.5 13a9 9 0 1 0 2.6-6.4L3 9" />
              </svg>
              Restore
            </button>
            <button className="btn ghost sm">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" style={{ width: 15, height: 15 }}>
                <circle cx="11" cy="11" r="7" />
                <path d="M21 21l-4.3-4.3" />
              </svg>
              Re-scan
            </button>
            <button className="btn danger sm">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" style={{ width: 15, height: 15 }}>
                <path d="M3 6h18M8 6V4h8v2M19 6l-1 14H6L5 6" />
              </svg>
              Delete permanently
            </button>
          </div>
        </div>

        <table className="table">
          <thead>
            <tr>
              <th style={{ width: 36 }} />
              <th>File</th>
              <th>Threat</th>
              <th>Quarantined</th>
              <th>Status</th>
              <th style={{ textAlign: "right" }}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {ITEMS.map((it, i) =>
              removed.has(i) ? null : (
                <tr key={it.file + i}>
                  <td>
                    <input type="checkbox" className="check" checked={selected.has(i)} onChange={() => toggle(i)} aria-label={`Select ${it.file}`} />
                  </td>
                  <td>
                    <div className="cell-stack">
                      <span className="q-name">{it.file}</span>
                      <span className="q-sub">{it.origin} · {it.size}</span>
                    </div>
                  </td>
                  <td>
                    <span className={"pill " + it.severity} style={{ fontSize: 10 }}>
                      <span className="dot" />
                      {it.threat}
                    </span>
                  </td>
                  <td className="mono sub">{it.when}</td>
                  <td>
                    <span className="lock">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <rect x="3" y="11" width="18" height="10" rx="2" />
                        <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                      </svg>
                      encrypted
                    </span>
                  </td>
                  <td style={{ textAlign: "right" }}>
                    <div className="flex gap-8" style={{ justifyContent: "flex-end" }}>
                      <button className="btn ghost sm" title="Restore">Restore</button>
                      <button className="btn danger sm" title="Delete" onClick={() => deleteRow(i)}>Delete</button>
                    </div>
                  </td>
                </tr>
              )
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
