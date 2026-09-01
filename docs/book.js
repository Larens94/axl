(() => {
  const BOOK_PAGE = "book.html";
  const params = new URLSearchParams(window.location.search);
  const contentEl = document.getElementById("content");
  const sidebarEl = document.getElementById("sidebar");
  const statusEl = document.getElementById("status");
  const searchEl = document.getElementById("search");
  const toggleEl = document.getElementById("menu-toggle");

  let nav = null;
  let currentPath = null;

  function pageUrl(path) {
    if (!path || path.startsWith("http://") || path.startsWith("https://")) {
      return path;
    }
    if (path.endsWith(".html")) {
      return path;
    }
    return `${BOOK_PAGE}?p=${encodeURIComponent(path)}`;
  }

  function setStatus(message, isError = false) {
    if (!statusEl) return;
    statusEl.textContent = message;
    statusEl.classList.toggle("error", isError);
  }

  function flattenItems(sections) {
    return sections.flatMap((section) => section.items || []);
  }

  function renderSidebar(sections, activePath) {
    if (!sidebarEl) return;
    sidebarEl.innerHTML = sections
      .map((section) => {
        const links = (section.items || [])
          .map((item) => {
            const href = item.external
              ? item.path
              : pageUrl(item.path);
            const active =
              !item.external && !item.navigate && item.path === activePath ? " active" : "";
            const target = item.external ? ' target="_blank" rel="noopener"' : "";
            return `<a class="book-nav-link${active}" href="${href}" data-path="${item.path ?? ""}" data-navigate="${item.navigate ? "1" : "0"}"${target}>${item.title}</a>`;
          })
          .join("");
        return `<div class="book-section"><p class="book-section-title">${section.title}</p>${links}</div>`;
      })
      .join("");
  }

  function resolvePath(basePath, href) {
    if (!href || href.startsWith("http://") || href.startsWith("https://") || href.startsWith("#")) {
      return href;
    }
    const dir = basePath.includes("/") ? basePath.slice(0, basePath.lastIndexOf("/") + 1) : "";
    const out = [];
    for (const part of `${dir}${href}`.split("/")) {
      if (!part || part === ".") continue;
      if (part === "..") out.pop();
      else out.push(part);
    }
    return out.join("/");
  }

  function enhanceMarkdownLinks(root, basePath) {
    root.querySelectorAll("a[href]").forEach((anchor) => {
      const href = anchor.getAttribute("href");
      if (!href || href.startsWith("#") || href.startsWith("http")) return;
      if (href.endsWith(".md")) {
        const resolved = resolvePath(basePath, href);
        anchor.setAttribute("href", pageUrl(resolved));
        anchor.dataset.path = resolved;
      }
    });
  }

  async function loadMarkdown(path) {
    const response = await fetch(path);
    if (!response.ok) {
      throw new Error(`Impossibile caricare ${path} (${response.status})`);
    }
    const text = await response.text();
    const html = marked.parse(text, { mangle: false, headerIds: true });
    contentEl.innerHTML = `<article class="book-article">${html}</article>`;
    enhanceMarkdownLinks(contentEl, path);
    document.title = `${path} · AXL 4 Docs`;
    setStatus(path);
  }

  async function showPage(path) {
    if (!path) path = nav?.default || "README.md";
    currentPath = path;
    renderSidebar(nav.sections, path);
    contentEl.innerHTML = `<article class="book-article"><p>Caricamento…</p></article>`;
    const item = flattenItems(nav.sections).find((entry) => entry.path === path);
    try {
      if (item?.external || item?.navigate) {
        window.location.href = pageUrl(path);
        return;
      }
      await loadMarkdown(path);
      history.replaceState({ path }, "", pageUrl(path));
    } catch (error) {
      contentEl.innerHTML = `<article class="book-article"><h1>Errore</h1><p>${error.message}</p><p>Esegui <code>sh scripts/prepare-docs-site.sh</code> poi <code>sh scripts/serve-docs.sh</code>, oppure attendi il deploy GitHub Pages.</p></article>`;
      setStatus("Errore caricamento", true);
    }
  }

  function bindSearch() {
    if (!searchEl) return;
    searchEl.addEventListener("input", () => {
      const query = searchEl.value.trim().toLowerCase();
      sidebarEl.querySelectorAll(".book-nav-link").forEach((link) => {
        const text = link.textContent.toLowerCase();
        link.style.display = text.includes(query) ? "block" : "none";
      });
    });
  }

  function bindContentLinks() {
    contentEl.addEventListener("click", (event) => {
      const link = event.target.closest("a");
      if (!link || link.target === "_blank") return;
      const path = link.dataset.path;
      if (!path || !path.endsWith(".md")) return;
      event.preventDefault();
      showPage(path);
    });
  }

  function bindSidebarLinks() {
    sidebarEl.addEventListener("click", (event) => {
      const link = event.target.closest("a.book-nav-link");
      if (!link || link.target === "_blank") return;
      if (link.dataset.navigate === "1") return;
      const path = link.getAttribute("data-path");
      if (!path || !path.endsWith(".md")) return;
      event.preventDefault();
      showPage(path);
      sidebarEl.classList.remove("open");
    });
  }

  window.addEventListener("popstate", (event) => {
    const path = event.state?.path || params.get("p") || nav?.default;
    showPage(path);
  });

  toggleEl?.addEventListener("click", () => {
    sidebarEl.classList.toggle("open");
  });

  async function init() {
    const response = await fetch("nav.json");
    nav = await response.json();
    bindSearch();
    bindSidebarLinks();
    bindContentLinks();
    const initial = params.get("p") || nav.default || "README.md";
    await showPage(initial);
  }

  if (typeof marked === "undefined") {
    contentEl.innerHTML =
      "<article class='book-article'><h1>Marked.js mancante</h1><p>Controlla la connessione CDN.</p></article>";
    return;
  }

  init().catch((error) => {
    contentEl.innerHTML = `<article class="book-article"><h1>Errore inizializzazione</h1><p>${error.message}</p></article>`;
    setStatus("nav.json non caricato", true);
  });
})();
