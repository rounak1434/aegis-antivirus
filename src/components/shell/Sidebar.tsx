import { NavLink } from "react-router-dom";
import { Icon } from "../Icon";
import { NAV } from "./nav";

/**
 * Grouped sidebar, ported from design-prototype/js/shell.js `buildSidebar()`.
 * The active item is derived from the route via NavLink's isActive, replacing
 * the prototype's `data-active` string match.
 */
export function Sidebar() {
  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="logo">
          <Icon name="shield" strokeWidth={1.6} />
        </div>
        <div>
          <div className="bn">Aegis</div>
          <div className="bv">v0.4.0 · open source</div>
        </div>
      </div>

      {NAV.map((section) => (
        <div key={section.group}>
          <div className="nav-label">{section.group}</div>
          {section.items.map((item) => (
            <NavLink
              key={item.key}
              to={item.path}
              end={item.path === "/"}
              className={({ isActive }) => "nav-item" + (isActive ? " active" : "")}
            >
              <Icon name={item.icon} />
              <span>{item.label}</span>
              {item.badge ? <span className="badge num">{item.badge}</span> : null}
            </NavLink>
          ))}
        </div>
      ))}

      <div className="spacer" />
      <div className="shield-mini">
        <Icon name="shield" />
        <div>
          <div className="sm-t">Protected</div>
          <div className="sm-s">Real-time on · defs 2h ago</div>
        </div>
      </div>
    </aside>
  );
}
