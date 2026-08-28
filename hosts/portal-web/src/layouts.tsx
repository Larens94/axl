import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import {
  axlDefaultApp,
  axlLayoutSlots,
  type AxlLayout,
} from "./generated/axl_layouts";
import { axlProductionRoutes } from "./generated/axl_routes";

type Props = {
  layout: AxlLayout;
  path: string;
  children: ReactNode;
};

function navFor(layout: AxlLayout) {
  return axlProductionRoutes.filter((route) => {
    if (route.kind !== "page") return false;
    if (layout === "GuestLayout") {
      return route.layout === "GuestLayout";
    }
    if (layout === "AdminLayout") {
      return route.layout === "AdminLayout";
    }
    return route.layout === "AppLayout";
  });
}

export function LayoutShell({ layout, path, children }: Props) {
  const links = navFor(layout);
  return (
    <div className={`shell shell-${layout}`} data-layout-slot={axlLayoutSlots[layout]}>
      <header className="shell-header">
        <Link to="/" className="brand">
          {axlDefaultApp}
        </Link>
        <span className="layout-tag">{layout}</span>
      </header>
      <div className="shell-body">
        <aside className="shell-nav">
          <p className="nav-label">Routes (axl-ui/1)</p>
          <ul>
            {links.map((route) => (
              <li key={route.path}>
                <Link
                  to={route.path}
                  className={route.path === path ? "active" : undefined}
                >
                  {route.path}
                </Link>
              </li>
            ))}
          </ul>
        </aside>
        <main className="shell-main">{children}</main>
      </div>
    </div>
  );
}
