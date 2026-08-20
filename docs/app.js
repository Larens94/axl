(() => {
  const menu = document.querySelector('#menu');
  const mobile = document.querySelector('#mobile-nav');
  menu?.addEventListener('click', () => {
    const open = menu.getAttribute('aria-expanded') !== 'true';
    menu.setAttribute('aria-expanded', String(open));
    mobile.hidden = !open;
    mobile.classList.toggle('open', open);
  });
  mobile?.querySelectorAll('a').forEach((link) => link.addEventListener('click', () => {
    menu?.setAttribute('aria-expanded', 'false');
    mobile.hidden = true;
    mobile.classList.remove('open');
  }));

  document.querySelectorAll('.source-tab').forEach((tab) => tab.addEventListener('click', () => {
    document.querySelectorAll('.source-tab').forEach((item) => item.classList.remove('active'));
    document.querySelectorAll('.source-panel').forEach((item) => item.classList.remove('active'));
    tab.classList.add('active');
    document.getElementById(tab.dataset.source)?.classList.add('active');
  }));

  document.querySelectorAll('.cmd-tab').forEach((tab) => tab.addEventListener('click', () => {
    document.querySelectorAll('.cmd-tab').forEach((item) => item.classList.remove('active'));
    document.querySelectorAll('.cmd-panel').forEach((item) => item.classList.remove('active'));
    tab.classList.add('active');
    document.getElementById(tab.dataset.cmd)?.classList.add('active');
  }));

  document.querySelectorAll('[data-copy]').forEach((button) => button.addEventListener('click', async () => {
    const target = document.getElementById(button.dataset.copy);
    if (!target) return;
    await navigator.clipboard.writeText(target.innerText);
    const label = button.textContent;
    button.textContent = 'Copiato';
    setTimeout(() => { button.textContent = label; }, 1200);
  }));
})();
