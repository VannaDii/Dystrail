/* Runs before the WASM entry point; never writes or clears a player's saves. */
(() => {
  const root = new URL('../', document.currentScript.src);
  // The entry URL must be inside the worker scope before installation starts.
  if (root.pathname !== '/' && location.pathname === root.pathname.slice(0, -1)) {
    window.dystrailLaunch = new Promise(() => {});
    location.replace(root.pathname + location.search + location.hash);
    return;
  }
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
  async function prepareFonts(result) {
    const definitions = [];
    const collect = (owner, href) => {
      for (const rule of owner.cssRules) {
        if (rule.type === CSSRule.FONT_FACE_RULE) definitions.push({owner,rule,href});
        else if (rule.styleSheet) collect(rule.styleSheet,rule.styleSheet.href);
        else if (rule.cssRules) collect(rule,href);
      }
    };
    for (const sheet of document.styleSheets) collect(sheet,sheet.href || document.baseURI);
    const cache = await caches.open(result.cache);
    const used = new Set();
    const fonts = await timeout(Promise.all(definitions.map(async ({rule,href}) => {
      const source = rule.style.getPropertyValue('src');
      const urls = [...source.matchAll(/url\(\s*(?:"([^"]+)"|'([^']+)'|([^\s)]+))\s*\)/g)];
      if (urls.length !== 1 || /\blocal\s*\(/i.test(source)) throw new Error('Font must have one bundled source');
      const url = new URL(urls[0][1] || urls[0][2] || urls[0][3],href);
      const asset = result.assets.find(item => new URL(item.path,root).href === url.href);
      if (!asset || url.origin !== root.origin || !/\.(woff2?|ttf|otf)$/i.test(asset.path)) throw new Error('Font missing from the prepared release');
      used.add(asset.path);
      const response = await cache.match(url);
      if (!response) throw new Error('Font missing from the prepared cache');
      const bytes = await response.arrayBuffer();
      const digest = await crypto.subtle.digest('SHA-256',bytes);
      const integrity = 'sha256-' + btoa(String.fromCharCode(...new Uint8Array(digest)));
      if (bytes.byteLength !== asset.bytes || integrity !== asset.integrity) {
        await cache.delete(url);
        await cache.delete(new URL('__offline_ready__',root));
        throw new Error('Incomplete font');
      }
      const descriptors = {};
      for (const [css,key] of [['font-style','style'],['font-weight','weight'],['font-stretch','stretch'],['unicode-range','unicodeRange'],['font-display','display'],['font-feature-settings','featureSettings'],['font-variation-settings','variationSettings']]) {
        const value = rule.style.getPropertyValue(css);
        if (value) descriptors[key] = value;
      }
      const family = rule.style.getPropertyValue('font-family').replace(/^(['"])(.*)\1$/,'$2');
      const face = new FontFace(family,bytes,descriptors);
      await face.load();
      if (face.status !== 'loaded') throw new Error('Font cannot be rendered');
      return face;
    })),15000);
    const assets = result.assets.filter(asset => /\.(woff2?|ttf|otf)$/i.test(asset.path));
    if (assets.some(asset => !used.has(asset.path))) throw new Error('Bundled font has no declaration');
    // Buffer-backed faces stay ready for unused bold, italic and RTL text too.
    // Remove equivalent URL-backed rules so a later scene cannot start a font request.
    for (const face of fonts) document.fonts.add(face);
    for (const {owner,rule} of definitions) {
      const index = Array.from(owner.cssRules).indexOf(rule);
      if (index < 0) throw new Error('Font declaration changed during preparation');
      owner.deleteRule(index);
    }
    await document.fonts.ready;
    window.dystrailPreparedFonts = Object.freeze(fonts);
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
    // Keep the decoded sources alive. Scene changes reuse these exact local URLs.
    window.dystrailPreparedImages = images;
    window.dystrailAssetUrls = Object.freeze(urls);
    const icon = document.querySelector('link[rel="icon"]');
    if (icon && urls['static/img/app-icon-192.png']) icon.href = urls['static/img/app-icon-192.png'];
    const personaArt = urls['static/img/journey/personas.png'];
    if (personaArt) document.documentElement.style.setProperty('--persona-art',`url("${personaArt}")`);
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
      await prepareFonts(prepared);
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
