(() => {
  function prepareNavigation() {
    const content = document.getElementById('mdbook-content');
    const toggle = document.getElementById('mdbook-sidebar-toggle');
    const anchor = document.getElementById('mdbook-sidebar-toggle-anchor');
    if (!content || !toggle || !anchor) return;

    content.tabIndex = -1;
    const skip = document.createElement('a');
    skip.className = 'guide-skip';
    skip.href = '#mdbook-content';
    skip.textContent = 'Skip to guide content';
    skip.addEventListener('click', () => requestAnimationFrame(() => content.focus()));
    document.body.prepend(skip);

    toggle.setAttribute('role', 'button');
    toggle.tabIndex = 0;
    toggle.addEventListener('keydown', (event) => {
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        toggle.click();
      }
    });
    anchor.addEventListener('change', () => {
      toggle.setAttribute('aria-expanded', String(anchor.checked));
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', prepareNavigation, { once: true });
  } else {
    prepareNavigation();
  }
})();
