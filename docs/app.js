(() => {
  const root = document.documentElement;
  const themeButton = document.querySelector('#theme-toggle');
  const storedTheme = localStorage.getItem('axl-theme');
  const preferredDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

  if (storedTheme === 'dark' || (!storedTheme && preferredDark)) {
    root.dataset.theme = 'dark';
  }

  themeButton?.addEventListener('click', () => {
    const next = root.dataset.theme === 'dark' ? 'light' : 'dark';
    root.dataset.theme = next;
    localStorage.setItem('axl-theme', next);
  });

  const menuButton = document.querySelector('#menu-toggle');
  const mobileNav = document.querySelector('#mobile-nav');
  menuButton?.addEventListener('click', () => {
    const open = menuButton.getAttribute('aria-expanded') !== 'true';
    menuButton.setAttribute('aria-expanded', String(open));
    mobileNav.hidden = false;
    mobileNav.classList.toggle('open', open);
    if (!open) mobileNav.hidden = true;
  });
  mobileNav?.querySelectorAll('a').forEach((link) => link.addEventListener('click', () => {
    menuButton?.setAttribute('aria-expanded', 'false');
    mobileNav.classList.remove('open');
    mobileNav.hidden = true;
  }));

  document.querySelectorAll('.copy-button').forEach((button) => {
    button.addEventListener('click', async () => {
      const target = document.getElementById(button.dataset.copyTarget);
      if (!target) return;
      await navigator.clipboard.writeText(target.innerText);
      const old = button.textContent;
      button.textContent = 'Copiato';
      setTimeout(() => { button.textContent = old; }, 1400);
    });
  });

  document.querySelectorAll('.command-tab').forEach((tab) => {
    tab.addEventListener('click', () => {
      document.querySelectorAll('.command-tab').forEach((item) => item.classList.remove('active'));
      document.querySelectorAll('.command-content').forEach((panel) => panel.classList.remove('active'));
      tab.classList.add('active');
      document.querySelector(`[data-panel="${tab.dataset.tab}"]`)?.classList.add('active');
    });
  });

  const observer = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        entry.target.classList.add('visible');
        observer.unobserve(entry.target);
      }
    });
  }, { threshold: 0.12 });
  document.querySelectorAll('.reveal').forEach((element) => observer.observe(element));
})();
