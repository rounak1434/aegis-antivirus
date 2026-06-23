import { useLocation } from "react-router-dom";
import { Icon } from "./Icon";
import { navItemByPath } from "./shell/nav";

/**
 * Interim view for screens not yet converted in the current migration slice.
 * This is a real, functional component (not a stub of the target feature): it
 * renders inside the production shell using the prototype's design system and
 * states clearly which screen is queued next. Each converted screen replaces
 * this with its real feature route.
 */
export function SectionComingNext() {
  const { pathname } = useLocation();
  const nav = navItemByPath(pathname);
  const label = nav?.label ?? "This section";

  return (
    <div className="card pad-lg rise" style={{ maxWidth: 560 }}>
      <div className="flex items-center gap-12 mb-16">
        <div className="si" style={{ width: 40, height: 40, borderRadius: 10, display: "grid", placeItems: "center", background: "var(--accent-wash)", border: "1px solid var(--accent-dim)", color: "var(--accent)" }}>
          <Icon name={nav?.icon ?? "shield"} size={20} />
        </div>
        <div>
          <h3 style={{ fontSize: 15 }}>{label}</h3>
          <div className="muted" style={{ fontSize: 12.5, marginTop: 2 }}>
            Converted from the prototype in the next migration step.
          </div>
        </div>
      </div>
      <div className="divider" />
      <p style={{ fontSize: 12.5, color: "var(--fg-2)", lineHeight: 1.55 }}>
        The Dashboard is the first converted screen. The remaining prototype
        pages are being ported one by one with full visual parity, following the
        screen map in <span className="kbd">PROTOTYPE_AUDIT.md</span>.
      </p>
    </div>
  );
}
