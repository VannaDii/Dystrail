/* Runs before the WASM entry point; never writes or clears a player's saves. */
(() => {
  const root = new URL('../', document.currentScript.src);
  const build = document.querySelector('meta[name="dystrail-build"]')?.content;
  let language = (document.documentElement.lang || 'en').split('-')[0];
  try { language = (localStorage.getItem('dystrail.locale') || language).split('-')[0]; } catch { /* The saved copy still starts when preference storage is unavailable. */ }
  const copy = window.dystrailOfflineCopy?.[language] || window.dystrailOfflineCopy?.en || {};
  let installPrompt;
  window.dystrailInstalled = window.matchMedia('(display-mode: standalone)').matches || navigator.standalone === true;
  window.dystrailInstallAvailable = false;
  let registration;
  let launchPending = true;
  function publish(detail) {
    if (launchPending && detail.state === 'ready') detail = {...detail,state:'preparing_assets',percent:95};
    window.dystrailOffline = detail;
    window.dispatchEvent(new Event('dystrail-offline'));
    if (!launchPending) return;
    const label = document.getElementById('launch-message');
    if (label) label.textContent = (copy[detail.state] || copy.checking || 'Dystopian Trail').replace('{percent}', detail.percent || 0);
    const progress = document.getElementById('launch-progress');
    if (progress) {
      progress.value = detail.percent || 0;
      progress.hidden = ['error','unavailable'].includes(detail.state);
    }
  }
  function gate() {
    const section = document.createElement('section');
    section.id = 'launch-gate';
    section.lang = language;
    section.dir = language === 'ar' ? 'rtl' : 'ltr';
    section.setAttribute('aria-labelledby','launch-title');
    const body = document.createElement('div');
    const title = document.createElement('h1');
    title.textContent = 'Dystopian Trail';
    title.id = 'launch-title';
    const heading = document.createElement('h2');
    heading.textContent = copy.preparing;
    const message = document.createElement('p');
    message.id = 'launch-message';
    message.setAttribute('role','status');
    const progress = document.createElement('progress');
    progress.id = 'launch-progress';
    progress.max = 100;
    progress.value = 0;
    progress.setAttribute('aria-labelledby','launch-message');
    const help = document.createElement('p');
    help.id = 'launch-help';
    help.textContent = copy.launch_help;
    const retry = document.createElement('button');
    retry.id = 'launch-retry';
    retry.textContent = copy.retry;
    retry.hidden = true;
    retry.addEventListener('click', () => location.reload());
    body.append(title,heading,message,progress,help,retry);
    section.append(body);
    document.body.prepend(section);
  }
  async function timeout(promise, ms) {
    let timer;
    try { return await Promise.race([promise, new Promise((_,reject) => { timer = setTimeout(() => reject(new Error('timeout')),ms); })]); }
    finally { clearTimeout(timer); }
  }
  async function installed(worker) {
    let check;
    try {
      await timeout(new Promise((resolve,reject) => {
        check = () => {
          if (['installed','activating','activated'].includes(worker.state)) resolve();
          else if (worker.state === 'redundant') reject(new Error('install failed'));
        };
        worker.addEventListener('statechange',check);
        check();
      }),900000);
    } finally { worker.removeEventListener('statechange',check); }
  }
  async function activate(worker) {
    let check;
    try {
      await timeout(new Promise(resolve => {
        check = () => { if (navigator.serviceWorker.controller === worker) resolve(); };
        navigator.serviceWorker.addEventListener('controllerchange',check);
        check();
        worker.postMessage({type:'activate-update'});
      }),10000);
    } finally { navigator.serviceWorker.removeEventListener('controllerchange',check); }
  }
  async function prepare(worker) {
    const channel = new MessageChannel();
    try {
      const result = await timeout(new Promise(resolve => {
        channel.port1.onmessage = event => resolve(event.data);
        worker.postMessage({type:'prepare-offline'},[channel.port2]);
      }),900000);
      if (result.state !== 'ready') throw new Error('cache incomplete');
      return result;
    } finally { channel.port1.close(); channel.port2.close(); }
  }
  async function prepareArtwork(result) {
    const cache = await caches.open(result.cache);
    const assets = result.assets.filter(asset => /\.(png|jpe?g|webp|gif|svg|ico)$/i.test(asset.path));
    const queue = [...assets], urls = Object.create(null), images = [];
    let decoded = 0;
    const failures = await Promise.allSettled(Array.from({length:4},async () => {
      while (queue.length) {
        const asset = queue.shift();
        const response = await cache.match(new URL(asset.path,root));
        if (!response) throw new Error('Artwork missing from the prepared release');
        const bytes = await response.arrayBuffer();
        if (bytes.byteLength !== asset.bytes) throw new Error('Incomplete artwork');
        const extension = asset.path.split('.').pop().toLowerCase();
        const type = extension === 'svg' ? 'image/svg+xml' : extension === 'ico' ? 'image/x-icon' : extension === 'jpg' ? 'image/jpeg' : `image/${extension}`;
        const url = URL.createObjectURL(new Blob([bytes],{type})) + '#' + encodeURIComponent(asset.path);
        urls[asset.path] = url;
        const image = new Image();
        image.decoding = 'sync';
        image.src = url;
        await image.decode();
        if (!image.naturalWidth || !image.naturalHeight) throw new Error('Artwork cannot be rendered');
        images.push(image);
        decoded++;
        publish({state:'preparing_assets',percent:95+Math.floor(decoded*5/assets.length),revision:result.revision});
      }
    }));
    if (failures.some(result => result.status === 'rejected')) {
      for (const url of Object.values(urls)) URL.revokeObjectURL(url.split('#')[0]);
      throw new Error('Artwork preparation failed');
    }
    await document.fonts.ready;
    // Keep the decoded sources alive. Scene changes reuse these exact local URLs.
    window.dystrailPreparedImages = images;
    window.dystrailAssetUrls = Object.freeze(urls);
    const icon = document.querySelector('link[rel="icon"]');
    if (icon && urls['static/img/app-icon-192.png']) icon.href = urls['static/img/app-icon-192.png'];
    for (const [variable,path] of [['--persona-art','static/img/journey/personas.png'],['--van-art','static/img/journey/van-crew.png']]) {
      if (urls[path]) document.documentElement.style.setProperty(variable,`url("${urls[path]}")`);
    }
  }
  const launch = async () => {
    gate();
    publish({state:'checking',percent:0,build});
    if (!('serviceWorker' in navigator) || !window.isSecureContext) {
      publish({state:'unavailable',percent:0});
      document.getElementById('launch-retry').hidden = false;
      return new Promise(() => {});
    }
    navigator.serviceWorker.addEventListener('message', event => {
      if (event.data?.type === 'dystrail-offline') publish(event.data);
    });
    try {
      registration = await navigator.serviceWorker.getRegistration(root);
      if (!registration) registration = await timeout(navigator.serviceWorker.register(new URL('sw.js',root),{scope:root.pathname,updateViaCache:'none'}),10000);
      const previous = registration.active;
      try {
        if (navigator.onLine && previous) await timeout(registration.update(),8000);
        const next = registration.installing || registration.waiting;
        if (next) {
          publish({state:previous ? 'updating' : 'downloading',percent:0});
          await installed(next);
          await activate(next);
          if (previous) {
            // Only a complete new version replaces the HTML and WASM together.
            location.reload();
            return new Promise(() => {});
          }
        }
      } catch (error) {
        // A failed update may retain the previous complete build, never a partial one.
        if (!previous) throw error;
      }
      const active = registration.active;
      if (!active) throw new Error('no active offline copy');
      const prepared = await prepare(active);
      await activate(active);
      await prepareArtwork(prepared);
      navigator.storage?.persist?.().catch(() => {});
      launchPending = false;
      publish(prepared);
      document.getElementById('launch-gate').remove();
    } catch {
      publish({state:'error',percent:0});
      document.getElementById('launch-help').textContent = copy.launch_error;
      document.getElementById('launch-retry').hidden = false;
      return new Promise(() => {});
    }
  };
  window.dystrailLaunch = launch();
  window.addEventListener('online', () => {
    if (launchPending) return;
    if (registration) registration.update().catch(() => {});
    else navigator.serviceWorker?.getRegistration(root).then(value => { registration = value; registration?.update().catch(() => {}); }).catch(() => {});
  });
  window.addEventListener('dystrail-offline-retry', () => location.reload());
  window.addEventListener('beforeinstallprompt',event => {
    event.preventDefault();
    if (window.dystrailInstalled) return;
    installPrompt = event;
    window.dystrailInstallAvailable = true;
    window.dispatchEvent(new Event('dystrail-offline'));
  });
  window.addEventListener('dystrail-install',async () => {
    if (!installPrompt) return;
    const prompt = installPrompt;
    installPrompt = undefined;
    window.dystrailInstallAvailable = false;
    window.dispatchEvent(new Event('dystrail-offline'));
    try { await prompt.prompt(); } catch { /* Browser installation guidance remains available. */ }
  });
  window.addEventListener('appinstalled',() => {
    window.dystrailInstalled = true;
    installPrompt = undefined;
    window.dystrailInstallAvailable = false;
    window.dispatchEvent(new Event('dystrail-offline'));
  });
})();
