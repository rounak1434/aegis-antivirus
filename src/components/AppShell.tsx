import { Outlet, useLocation } from "react-router-dom";
import { WinBar } from "./shell/WinBar";
import { Sidebar } from "./shell/Sidebar";
import { TopBar } from "./shell/TopBar";
import { navItemByPath } from "./shell/nav";

/**
 * Layout route. Reproduces the prototype's shell.js composition:
 *   winbar + .app(has-winbar) [ sidebar | main( topbar + content ) ]
 * Per-route title/crumb/wide come from the NAV metadata, replacing the
 * prototype's per-page `data-*` attributes. Nested routes render into <Outlet>.
 */
export function AppShell() {
  const { pathname } = useLocation();
  const nav = navItemByPath(pathname);
  const title = nav?.title ?? "Aegis";
  const crumb = nav?.crumb;
  const wide = nav?.wide ?? false;

  return (
    <>
      <WinBar />
      <div className="app has-winbar">
        <Sidebar />
        <div className="main">
          <TopBar title={title} crumb={crumb} />
          <div className="content-scroll">
            <div className={"content" + (wide ? " wide" : "")}>
              <Outlet />
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
