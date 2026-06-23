import { Icon } from "../Icon";

/**
 * Sticky top bar, ported from design-prototype/js/shell.js `buildTopbar()`.
 * Title + breadcrumb come from the active route's nav metadata.
 */
export function TopBar({ title, crumb }: { title: string; crumb?: string }) {
  return (
    <header className="topbar">
      {crumb ? <span className="crumb">{crumb}</span> : null}
      <h1>{title}</h1>
      <div className="tb-right">
        <button className="icon-btn" title="Search (Ctrl K)" aria-label="Search">
          <Icon name="search" />
        </button>
        <button className="icon-btn" title="Notifications" aria-label="Notifications">
          <Icon name="bell" />
        </button>
        <span className="pill ok">
          <span className="dot" />
          Protected
        </span>
      </div>
    </header>
  );
}
