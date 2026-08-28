import { Navigate, Route, Routes, useLocation } from "react-router-dom";
import {
  axlApp,
  axlProductionRoutes,
  type AxlRoute,
} from "./generated/axl_routes";
import { LayoutShell } from "./layouts";
import { AxlSurface } from "./AxlSurface";

function routeElement(route: AxlRoute) {
  return (
    <LayoutShell layout={route.layout} path={route.path}>
      <AxlSurface route={route} />
    </LayoutShell>
  );
}

export function App() {
  const location = useLocation();
  return (
    <div className="host" data-axl-app={axlApp} data-path={location.pathname}>
      <Routes>
        {axlProductionRoutes.map((route) => (
          <Route
            key={`${route.kind}:${route.path}`}
            path={route.path === "/" ? "/" : route.path}
            element={routeElement(route)}
          />
        ))}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </div>
  );
}
