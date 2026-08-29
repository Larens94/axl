import { useEffect, useState } from "react";
import type { AxlRoute } from "./generated/axl_routes";

type Props = { route: AxlRoute };

/**
 * Host surface: load AXL-rendered HTML/JSON through the Vite same-origin proxy.
 * No portal business rules live here — only fetch + display.
 */
export function AxlSurface({ route }: Props) {
  const [html, setHtml] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setPending(true);
    setError(null);
    const path = route.path === "" ? "/" : route.path;
    fetch(path, {
      credentials: "include",
      headers: { Accept: "text/html, application/json;q=0.9,*/*;q=0.8" },
    })
      .then(async (response) => {
        const contentType = response.headers.get("content-type") ?? "";
        const body = await response.text();
        if (!response.ok) {
          throw new Error(`${response.status}: ${body.slice(0, 200)}`);
        }
        if (cancelled) return;
        if (contentType.includes("text/html")) {
          setHtml(body);
        } else {
          setHtml(
            `<pre class="json-fallback">${escapeHtml(body)}</pre>`,
          );
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
          setHtml("");
        }
      })
      .finally(() => {
        if (!cancelled) setPending(false);
      });
    return () => {
      cancelled = true;
    };
  }, [route.path, route.kind]);

  if (pending) {
    return <p className="status">Loading AXL surface…</p>;
  }
  if (error) {
    return (
      <div className="status error">
        <p>Backend unreachable or route error.</p>
        <code>{error}</code>
        <p className="hint">
          Start AXL with <code>./scripts/demo-portal.sh</code> then{" "}
          <code>npm run dev</code> in <code>hosts/portal-web</code>.
        </p>
      </div>
    );
  }

  return (
    <div
      className="axl-surface"
      dangerouslySetInnerHTML={{ __html: extractMain(html) }}
    />
  );
}

function extractMain(html: string): string {
  const main = html.match(/<main[\s\S]*?<\/main>/i);
  if (main) return main[0];
  const body = html.match(/<body[^>]*>([\s\S]*)<\/body>/i);
  if (body) return body[1];
  return html;
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}
