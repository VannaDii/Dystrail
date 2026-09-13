// Focused update/rollback verification using Apple's native Safari WebDriver.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { createServer } from 'node:http';
import assert from 'node:assert/strict';

const built = process.env.PLAYTEST_DIST || '/tmp/dystrail-continuity-preview/play';
const output = process.env.SAFARI_OUTPUT || '/tmp/dystrail-safari-pwa';
mkdirSync(output, { recursive: true });
const original = JSON.parse(readFileSync(join(built, 'offline-manifest.json'), 'utf8'));
const html = readFileSync(join(built, 'index.html'), 'utf8');
const worker = readFileSync('../scripts/offline-worker.js', 'utf8');
let version = 'one', hold = true, fail = false, waiting = false, release = () => {};
const updatedHtml = () => html.replace('</head>', `<meta name="test-build" content="${version}"></head>`);
const build = () => ({ ...original, revision: `${original.revision}-${version}`, assets: original.assets.map(asset =>
  asset.path === 'index.html' ? { ...asset, bytes: Buffer.byteLength(updatedHtml()), integrity: 'sha256-' + createHash('sha256').update(updatedHtml()).digest('base64') } : asset) });
const server = createServer(async (req, res) => {
  const path = new URL(req.url, 'http://fixture').pathname.replace(/^\/play\//, '');
  res.setHeader('Cache-Control', 'no-store');
  if (path === 'sw.js') { res.setHeader('Content-Type', 'text/javascript'); res.end('const BUILD=' + JSON.stringify(build()) + ';\n' + worker); return; }
  if (path === 'static/img/journey/town-npcs-v1.png') {
    if (fail) { res.statusCode = 503; res.end('Deliberately interrupted update'); return; }
    if (hold) await new Promise(resolve => { waiting = true; release = resolve; });
  }
  if (!path || path === 'index.html' || !path.includes('.')) { res.setHeader('Content-Type', 'text/html'); res.end(updatedHtml()); return; }
  const mime = path.endsWith('.js') ? 'text/javascript' : path.endsWith('.wasm') ? 'application/wasm' : path.endsWith('.css') ? 'text/css' : path.endsWith('.webmanifest') ? 'application/manifest+json' : 'application/octet-stream';
  res.setHeader('Content-Type', mime);
  try { res.end(readFileSync(join(built, path))); } catch { res.statusCode = 404; res.end(); }
});
await new Promise(resolve => server.listen(Number(process.env.SAFARI_PORT || 0), '127.0.0.1', resolve));
const port = server.address().port, origin = `http://127.0.0.1:${port}/play/`;
let session;
async function call(path, body, method = body === undefined ? 'GET' : 'POST') {
  const response = await fetch('http://127.0.0.1:9515' + path, { method, signal: AbortSignal.timeout(30000), headers: { 'Content-Type': 'application/json' }, ...(body === undefined ? {} : { body: JSON.stringify(body) }) });
  const result = await response.json();
  if (!response.ok || result.value?.error) throw new Error(JSON.stringify(result.value));
  return result.value;
}
const cmd = (path, body, method) => call(`/session/${session}${path}`, body, method);
const exec = (script, args = []) => cmd('/execute/sync', { script, args });
async function until(check, timeout = 20000) {
  const start = Date.now();
  while (Date.now() - start < timeout) { if (await check()) return; await new Promise(resolve => setTimeout(resolve, 150)); }
  throw new Error('Native Safari condition timed out');
}
async function click(label, scroll = true) {
  console.log('Click', label);
  await until(() => exec('return [...document.querySelectorAll("button")].some(e=>e.offsetHeight&&!e.disabled&&(e.getAttribute("aria-label")===arguments[0]||e.textContent.trim()===arguments[0]));', [label]));
  const element = await exec('return [...document.querySelectorAll("button")].find(e=>e.offsetHeight&&!e.disabled&&(e.getAttribute("aria-label")===arguments[0]||e.textContent.trim()===arguments[0]));', [label]);
  assert(element, label);
  if (scroll) await exec('const r=arguments[0].getBoundingClientRect();if(r.top<0||r.bottom>innerHeight)arguments[0].scrollIntoView({block:"center",behavior:"instant"})', [element]);
  // Native Safari can discard input until it has painted the scrolled target.
  await new Promise(resolve => setTimeout(resolve, 150));
  await cmd('/element/' + element['element-6066-11e4-a52e-4f735466cecf'] + '/click', {});
}
async function fill(selector, text) {
  const element = await cmd('/element', { using: 'css selector', value: selector });
  const id = element['element-6066-11e4-a52e-4f735466cecf'];
  await cmd(`/element/${id}/clear`, {});
  await cmd(`/element/${id}/value`, { text });
}
// Inventory tags are a Rust HashSet; compare membership, not serialization order.
const state = () => exec('const s=JSON.parse(localStorage.getItem("dystrail.autosave.v1")).state;s.inventory.tags.sort();return s');
const ready = () => until(() => exec('return !!document.querySelector("#main") && window.dystrailOffline?.state==="ready"'), 40000);
const checks = [];
function pass(name) { checks.push(name); console.log('PASS', name); writeFileSync(join(output, 'validation.json'), JSON.stringify({ browser: 'Native Safari', sourceRevision: original.revision, origin, checks }, null, 2)); }
async function shot(name) {
  await cmd('/execute/async', { script: 'const done=arguments[arguments.length-1];requestAnimationFrame(()=>requestAnimationFrame(()=>done(true)));', args: [] });
  writeFileSync(join(output, name + '.png'), Buffer.from(await cmd('/screenshot'), 'base64'));
}
async function stopOrigin() { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); }

try {
  const created = await call('/session', { capabilities: { alwaysMatch: { browserName: 'safari' } } });
  session = created.sessionId;
  console.log('Safari', created.capabilities.browserVersion);
  await cmd('/timeouts', { script: 30000, pageLoad: 30000, implicit: 0 });
  await cmd('/window/rect', { width: 1280, height: 980 });
  await cmd('/url', { url: origin });
  await until(() => exec('return !!document.querySelector("#launch-gate")'));
  await until(() => waiting);
  assert.equal(await exec('return !!document.querySelector("#main")'), false);
  hold = false; release(); await ready();
  pass('First launch waits for its complete asset bundle');
  assert.equal(await exec('return document.visibilityState'), 'visible');
  await click('Choose your character'); await until(() => exec('return !!document.querySelector(".persona-select")'));
  await click('Journalist'); await click('Continue'); await until(() => exec('return !!document.querySelector("#crew-name")'));
  await fill('#crew-name-journalist', 'River Safari'); await fill('#crew-name', 'Safari Receipts');
  await click('Continue'); await click('Review & depart'); await click('Start the journey');
  await until(() => exec('return document.querySelector("#main").dataset.screen==="travel"'));
  const layout = () => exec('return {scroll:scrollY,boxes:[...document.querySelectorAll(".survival-hud, .hud-stat dt, .hud-stat dd, .journey-scene, .scene-status, .travel-toolbar, #main>footer, #game-menu-panel, .offline-status")].map(e=>{const r=e.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};})};');
  for (const width of [1280, 600]) {
    await cmd('/window/rect', { width, height: 980 });
    await exec('scrollTo({top:0,behavior:"instant"})'); await until(() => exec('return scrollY===0')); await click('Menu ▾', false);
    const before = await layout();
    await click('Help & tips', false);
    await until(() => exec('return document.querySelector(".help-switch").getAttribute("aria-checked")==="false"'));
    assert.deepEqual(await layout(), before);
    assert(await exec('return [...document.querySelectorAll("[data-help-kind=tip] button")].every(e=>getComputedStyle(e).visibility==="hidden"&&e.disabled)'));
    assert(await exec('return [...document.querySelectorAll("[data-help-kind=info] button")].every(e=>getComputedStyle(e).visibility==="visible"&&!e.disabled)'));
    await shot(`help-off-${width}`);
    await click('Help & tips', false);
    await until(() => exec('return document.querySelector(".help-switch").getAttribute("aria-checked")==="true"'));
    assert.deepEqual(await layout(), before); await click('Menu ▾', false);
  }
  pass('Help & tips toggles without moving stats, scene, controls or menu at desktop and narrow Safari widths; information remains usable');
  const saved = await state();

  await stopOrigin(); await cmd('/url', { url: origin + 'travel' }); await ready();
  assert.deepEqual(await state(), saved);
  const invalid = await cmd('/execute/async', { script: 'const done=arguments[arguments.length-1];Promise.all(arguments[0].map(async a=>{try{const r=await fetch(new URL(a.path,document.baseURI));const b=await r.arrayBuffer();const hash="sha256-"+btoa(String.fromCharCode(...new Uint8Array(await crypto.subtle.digest("SHA-256",b))));return r.ok&&b.byteLength===a.bytes&&hash===a.integrity?null:a.path;}catch{return a.path;}})).then(values=>done(values.filter(Boolean)),error=>done([String(error)]));', args: [build().assets] });
  assert.deepEqual(invalid, []); await shot('origin-stopped');
  pass(`With the origin stopped, the saved journey relaunches and all ${original.assets.length} cached files match their hashes`);

  version = 'two'; hold = true; waiting = false;
  await new Promise(resolve => server.listen(port, '127.0.0.1', resolve));
  const refresh = cmd('/refresh', {});
  await until(() => exec('return !!document.querySelector("#launch-gate")'));
  await until(() => waiting); assert.equal(await exec('return !!document.querySelector("#main")'), false);
  await shot('update-held'); hold = false; release(); await refresh;
  await until(() => exec('return document.querySelector("meta[name=test-build]")?.content==="two" && !!document.querySelector("#main")'), 40000);
  assert.deepEqual(await state(), saved);
  pass('A complete changed release replaces the old version before play and preserves the entire save');

  version = 'three'; fail = true; await cmd('/refresh', {}); await ready();
  assert.equal(await exec('return document.querySelector("meta[name=test-build]").content'), 'two');
  assert.deepEqual(await state(), saved); await shot('failed-update-fallback');
  pass('A failed update keeps the previous complete version and the entire save');
  await stopOrigin(); await cmd('/refresh', {}); await ready();
  assert.equal(await exec('return document.querySelector("meta[name=test-build]").content'), 'two');
  assert.deepEqual(await state(), saved);
  pass('After the interrupted update, the complete saved version still relaunches offline');
} catch (error) {
  await shot('failure').catch(() => {});
  console.error('Control geometry:', await exec('return [...document.querySelectorAll(".store-checkout-actions button")].map(e=>{const r=e.getBoundingClientRect();return {label:e.textContent,rect:{x:r.x,y:r.y,width:r.width,height:r.height},visible:innerHeight,hit:document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)?.outerHTML}})').catch(() => null));
  console.error('Safari page at failure:', await exec('return {visible:document.visibilityState,screen:document.querySelector("#main")?.dataset.screen,text:document.body.innerText.slice(0,5000)}').catch(() => null));
  throw error;
} finally {
  release();
  if (session) await cmd('', undefined, 'DELETE').catch(() => {});
  if (server.listening) await stopOrigin();
}
