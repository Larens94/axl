import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const axlTarget = process.env.AXL_PROXY_TARGET ?? "http://127.0.0.1:8080";

// Same-origin cookie proxy: browser → Vite → axl-compiler serve.
// Product routes/forms stay in AXL; this host only binds codegen layouts.
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      "^/(auth|clienti|prodotti|listini|preventivi|ordini|secure|jwt|rbac|api)(/|$)": {
        target: axlTarget,
        changeOrigin: true,
      },
      // HTML pages/forms rendered by AXL (session cookie stays on Vite origin).
      "^/(home|login|register|password-dimenticata|reimposta-password|admin)(/|$)": {
        target: axlTarget,
        changeOrigin: true,
      },
      "^/$": {
        target: axlTarget,
        changeOrigin: true,
        bypass(req) {
          // Keep SPA shell for Accept: text/html from the React app navigator.
          if (req.headers.accept?.includes("text/html") && req.url === "/") {
            return "/index.html";
          }
        },
      },
    },
  },
});
