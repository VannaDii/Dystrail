/* BUILD is generated from every shipped file by build_offline.py. */
const ROOT = self.registration.scope;
const PREFIX = `dystopian-trail:${new URL(ROOT).pathname}:`;
const CACHE = PREFIX + BUILD.revision;
const READY = new URL('__offline_ready__', ROOT).href;
let completedBytes = 0;
async function announce(state, target) {
  const message = {type:'dystrail-offline', state,
    percent:state === 'ready' ? 100 : Math.floor(completedBytes * 100 / BUILD.bytes)};
  if (target) target.postMessage(message);
  else for (const client of await self.clients.matchAll({includeUncontrolled:true})) {
    if (client.url.startsWith(ROOT)) client.postMessage(message);
  }
}
self.addEventListener('install', event => event.waitUntil((async () => {
  const cache = await caches.open(CACHE);
  const queue = [...BUILD.assets];
  try {
    await Promise.all(Array.from({length:6}, async () => {
      while (queue.length) {
        const asset = queue.shift();
        const url = new URL(asset.path, ROOT);
        const response = await fetch(url, {cache:'reload', integrity:asset.integrity});
        if (!response.ok) throw new Error(`Offline asset unavailable: ${asset.path}`);
        await cache.put(url, response);
        completedBytes += asset.bytes;
        await announce('downloading');
      }
    }));
    await cache.put(READY, new Response(BUILD.revision));
    await announce('ready');
    // An updated copy waits for the launch gate to request activation.
    if (!self.registration.active) await self.skipWaiting();
  } catch (error) {
    queue.length = 0;
    await announce('error');
    throw error;
  }
})()));
self.addEventListener('activate', event => event.waitUntil((async () => {
  for (const name of await caches.keys()) {
    const builds = (await caches.keys()).filter(key => key.startsWith(PREFIX));
    if (name.startsWith(PREFIX) && name !== CACHE && builds.indexOf(name) < builds.length - 2) await caches.delete(name);
  }
  await self.clients.claim();
  await announce('ready');
})()));
self.addEventListener('message', event => {
  if (event.data?.type === 'activate-update') { event.waitUntil(self.skipWaiting()); return; }
  if (event.data?.type !== 'offline-status') return;
  event.waitUntil((async () => {
    const cache = await caches.open(CACHE);
    await announce(await cache.match(READY) ? 'ready' : 'downloading', event.source);
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
