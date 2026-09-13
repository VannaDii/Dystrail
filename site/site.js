// Motion is an enhancement: content is visible before this script runs.
(() => {
  const motion = matchMedia('(prefers-reduced-motion: reduce)');
  if (motion.matches || !('IntersectionObserver' in window)) return;
  const observer = new IntersectionObserver(entries => {
    for (const entry of entries) {
      if (entry.isIntersecting) {
        entry.target.classList.add('is-revealed');
        observer.unobserve(entry.target);
      }
    }
  }, { threshold: 0.1 });
  document.querySelectorAll('[data-reveal]').forEach(element => observer.observe(element));
  motion.addEventListener('change', event => {
    if (event.matches) observer.disconnect();
  });
})();
