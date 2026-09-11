/* Runs before the WASM entry point; never writes or clears a player's saves. */
(() => {
  const root = new URL('../', document.currentScript.src);
  const build = document.querySelector('meta[name="dystrail-build"]')?.content;
  let language = (document.documentElement.lang || 'en').split('-')[0];
  try { language = (localStorage.getItem('dystrail.locale') || language).split('-')[0]; } catch { /* The saved copy still starts when preference storage is unavailable. */ }
  const copy = window.dystrailOfflineCopy?.[language] || window.dystrailOfflineCopy?.en || {};
  let installPrompt;
  let registration;
  let launchPending = true;
  function publish(detail) {
    window.dystrailOffline = detail;
    window.dispatchEvent(new Event('dystrail-offline'));
    const label = document.getElementById('launch-message');
    if (label && launchPending) label.textContent = (copy[detail.state] || copy.checking || 'Dystopian Trail').replace('{percent}', detail.percent || 0);
  }
  function gate() {
    if (document.getElementById('launch-gate')) return;
    const section = document.createElement('section');
    section.id = 'launch-gate';
    section.setAttribute('role','status');
    const title = document.createElement('h1');
    title.textContent = 'Dystopian Trail';
    const message = document.createElement('p');
    message.id = 'launch-message';
    message.textContent = copy.checking || 'Checking for updates…';
    section.append(title,message);
    document.body.prepend(section);
  }
  const timeout = (promise, ms) => Promise.race([promise, new Promise((_,reject) => setTimeout(() => reject(new Error('timeout')),ms))]);
  const installed = worker => new Promise((resolve,reject) => {
    const check = () => {
      if (['installed','activated'].includes(worker.state)) resolve();
      else if (worker.state === 'redundant') reject(new Error('install failed'));
    };
    worker.addEventListener('statechange',check);
    check();
  });
  const ask = () => (registration?.active || registration?.waiting || registration?.installing)?.postMessage({type:'offline-status'});
  const launch = async () => {
    if (!('serviceWorker' in navigator) || !window.isSecureContext) {
      publish({state:'unavailable',percent:0});
      return;
    }
    navigator.serviceWorker.addEventListener('message', event => {
      if (event.data?.type === 'dystrail-offline') publish(event.data);
    });
    try {
      if (!navigator.onLine && navigator.serviceWorker.controller) { navigator.serviceWorker.controller.postMessage({type:'offline-status'}); return; }
      registration = await timeout(navigator.serviceWorker.register(new URL('sw.js',root),{scope:root.pathname,updateViaCache:'none'}),8000);
      ask();
      if (navigator.onLine && registration.active) {
        gate();
        publish({state:'checking',percent:0});
        await timeout(registration.update(),8000);
        const next = registration.installing || registration.waiting;
        if (next) {
          publish({state:'updating',percent:0});
          await timeout(installed(next),180000);
          const changed = new Promise(resolve => navigator.serviceWorker.addEventListener('controllerchange',resolve,{once:true}));
          next.postMessage({type:'activate-update'});
          await timeout(changed,8000);
          // Reload the cached, complete new HTML before its WASM can start.
          location.reload();
          await new Promise(() => {});
        }
      }
      ask();
      navigator.storage?.persist?.().catch(() => {});
    } catch {
      publish({state:'error',percent:0});
    } finally {
      launchPending = false;
      document.getElementById('launch-gate')?.remove();
    }
  };
  publish({state:'downloading',percent:0,build});
  window.dystrailLaunch = launch();
  window.addEventListener('online', () => {
    if (launchPending) return;
    if (registration) registration.update().catch(() => {});
    else navigator.serviceWorker?.getRegistration(root).then(value => { registration = value; registration?.update().catch(() => {}); }).catch(() => {});
  });
  window.addEventListener('dystrail-offline-retry', () => location.reload());
  window.addEventListener('beforeinstallprompt',event => {
    event.preventDefault();
    installPrompt = event;
    window.dystrailInstallAvailable = true;
    window.dispatchEvent(new Event('dystrail-offline'));
  });
  window.addEventListener('dystrail-install',async () => {
    if (!installPrompt) return;
    await installPrompt.prompt();
    installPrompt = undefined;
    window.dystrailInstallAvailable = false;
    window.dispatchEvent(new Event('dystrail-offline'));
  });
  window.addEventListener('appinstalled',() => {
    window.dystrailInstalled = true;
    window.dispatchEvent(new Event('dystrail-offline'));
  });
})();
