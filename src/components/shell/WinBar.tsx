import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * Frameless Windows 11 title bar, ported from design-prototype/js/shell.js
 * `buildWinbar()`. The bar is the drag region; the min/max/close controls call
 * the real Tauri window API (no-ops gracefully outside a Tauri runtime, e.g. in
 * a plain browser preview).
 */
async function withWindow(action: "minimize" | "toggleMaximize" | "close") {
  try {
    const win = getCurrentWindow();
    if (action === "minimize") await win.minimize();
    else if (action === "toggleMaximize") await win.toggleMaximize();
    else await win.close();
  } catch {
    /* not running inside Tauri (browser preview) — ignore */
  }
}

export function WinBar() {
  return (
    <div className="winbar">
      <div className="wb-title">
        <span className="wb-dot" />
        Aegis Security — running with elevated privileges
      </div>
      <div className="win-ctrls">
        <button title="Minimize" onClick={() => void withWindow("minimize")} aria-label="Minimize">
          <svg width="11" height="11" viewBox="0 0 11 11" stroke="currentColor" strokeWidth="1.3">
            <line x1="1" y1="6" x2="10" y2="6" />
          </svg>
        </button>
        <button title="Maximize" onClick={() => void withWindow("toggleMaximize")} aria-label="Maximize">
          <svg width="11" height="11" viewBox="0 0 11 11" stroke="currentColor" strokeWidth="1.3" fill="none">
            <rect x="1.5" y="1.5" width="8" height="8" />
          </svg>
        </button>
        <button className="close" title="Close" onClick={() => void withWindow("close")} aria-label="Close">
          <svg width="11" height="11" viewBox="0 0 11 11" stroke="currentColor" strokeWidth="1.3">
            <line x1="1.5" y1="1.5" x2="9.5" y2="9.5" />
            <line x1="9.5" y1="1.5" x2="1.5" y2="9.5" />
          </svg>
        </button>
      </div>
    </div>
  );
}
