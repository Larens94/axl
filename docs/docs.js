(() => {
  const body = document.body;
  const language = body.dataset.lang || 'it';
  const dialog = document.querySelector('[data-search-dialog]');
  const input = document.querySelector('[data-search-input]');
  const results = document.querySelector('[data-search-results]');
  const sidebar = document.querySelector('[data-sidebar]');
  let index = [];

  const escapeHtml = (value) => value.replace(/[&<>'"]/g, (character) => ({
    '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;'
  })[character]);

  const loadIndex = async () => {
    if (index.length) return index;
    try {
      const response = await fetch('search-index.json');
      index = await response.json();
    } catch (_) {
      index = [];
    }
    return index;
  };

  const showResults = (query) => {
    const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
    const matches = !terms.length ? index.slice(0, 7) : index.filter((page) => {
      const haystack = `${page.title} ${page.label} ${page.text}`.toLocaleLowerCase();
      return terms.every((term) => haystack.includes(term));
    }).slice(0, 9);
    if (!matches.length) {
      results.innerHTML = `<p class="search-empty">${language === 'it' ? 'Nessun risultato' : 'No results found'}</p>`;
      return;
    }
    results.innerHTML = matches.map((page) => {
      const source = page.text.replace(/\s+/g, ' ').slice(0, 150);
      return `<a class="search-result" href="${escapeHtml(page.url)}"><b>${escapeHtml(page.title)}</b><p>${escapeHtml(source)}…</p></a>`;
    }).join('');
  };

  const openSearch = async () => {
    await loadIndex();
    showResults('');
    dialog?.showModal();
    setTimeout(() => input?.focus(), 0);
  };

  document.querySelectorAll('[data-open-search]').forEach((button) => {
    button.addEventListener('click', openSearch);
  });
  input?.addEventListener('input', () => showResults(input.value));
  dialog?.addEventListener('click', (event) => {
    if (event.target === dialog) dialog.close();
  });
  document.addEventListener('keydown', (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLocaleLowerCase() === 'k') {
      event.preventDefault();
      openSearch();
    }
    if (event.key === '/' && !/input|textarea/i.test(document.activeElement?.tagName || '')) {
      event.preventDefault();
      openSearch();
    }
  });
  const menuButton = document.querySelector('[data-menu]');
  menuButton?.addEventListener('click', (event) => {
    sidebar?.classList.toggle('open');
    event.currentTarget.setAttribute('aria-expanded', sidebar?.classList.contains('open') ? 'true' : 'false');
  });
  document.querySelectorAll('.sidebar a').forEach((link) => link.addEventListener('click', () => {
    sidebar?.classList.remove('open');
    menuButton?.setAttribute('aria-expanded', 'false');
  }));
})();
