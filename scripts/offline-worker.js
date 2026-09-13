/* BUILD is generated from every shipped file by build_offline.py. */
const ROOT = self.registration.scope;
const PREFIX = `dystopian-trail:${new URL(ROOT).pathname}:`;
const CACHE = PREFIX + BUILD.revision;
const READY = new URL('__offline_ready__', ROOT).href;
let completedBytes = 0;
let preparation;
function status(state) {
  return {type:'dystrail-offline', state, revision:BUILD.revision,
    percent:state === 'ready' ? 100 : Math.floor(completedBytes * 100 / BUILD.bytes)};
}
async function announce(state, target) {
  const message = status(state);
  if (target) target.postMessage(message);
  else for (const client of await self.clients.matchAll({includeUncontrolled:true})) {
    if (client.url.startsWith(ROOT)) client.postMessage(message);
  }
}
async function prepare() {
  const cache = await caches.open(CACHE);
  completedBytes = 0;
  const queue = [];
  // A readiness marker alone cannot detect an evicted or incomplete asset cache.
  for (const asset of BUILD.assets) {
    if (await cache.match(new URL(asset.path, ROOT))) completedBytes += asset.bytes;
    else queue.push(asset);
  }
  if (queue.length) {
    await cache.delete(READY);
    await announce('downloading');
    const downloads = await Promise.allSettled(Array.from({length:6}, async () => {
      while (queue.length) {
        const asset = queue.shift();
        try {
          const url = new URL(asset.path, ROOT);
          const response = await fetch(url, {cache:'reload', integrity:asset.integrity});
          if (!response.ok) throw new Error(`Offline asset unavailable: ${asset.path}`);
          await cache.put(url, response);
          completedBytes += asset.bytes;
          await announce('downloading');
        } catch (error) {
          queue.length = 0;
          throw error;
        }
      }
    }));
    if (downloads.some(result => result.status === 'rejected')) {
      await announce('error');
      throw new Error('Offline download incomplete');
    }
  }
  await cache.put(READY, new Response(BUILD.revision));
  await announce('ready');
}
function ensurePrepared() {
  preparation ||= prepare().finally(() => { preparation = undefined; });
  return preparation;
}
self.addEventListener('install', event => event.waitUntil((async () => {
  await ensurePrepared();
  // Updates wait for the launch gate so a running game keeps its current version.
  if (!self.registration.active) await self.skipWaiting();
})()));
self.addEventListener('activate', event => event.waitUntil((async () => {
  const builds = (await caches.keys()).filter(key => key.startsWith(PREFIX));
  for (const name of builds.slice(0,-2)) if (name !== CACHE) await caches.delete(name);
  await self.clients.claim();
  await announce('ready');
})()));
self.addEventListener('message', event => {
  if (event.data?.type === 'activate-update') { event.waitUntil(self.skipWaiting()); return; }
  if (!['offline-status','prepare-offline'].includes(event.data?.type)) return;
  event.waitUntil((async () => {
    let state = 'ready';
    try { await ensurePrepared(); } catch { state = 'error'; }
    await announce(state, event.source);
    event.ports[0]?.postMessage({...status(state), cache:CACHE, assets:BUILD.assets});
  })());
});
self.addEventListener('fetch', event => {
  if (event.request.method !== 'GET' || !event.request.url.startsWith(ROOT)) return;
  event.respondWith((async () => {
    const cache = await caches.open(CACHE);
    const key = event.request.mode === 'navigate' ? new URL('index.html', ROOT) : event.request;
    return await cache.match(key) || fetch(event.request);
  })());
});
