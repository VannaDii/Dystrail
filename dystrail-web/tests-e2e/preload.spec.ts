import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {join,resolve} from 'node:path';
import {createServer} from 'node:http';
import {savedState,importState,snap,waitForLaunch} from './helpers';
const built=process.env.PLAYTEST_DIST||resolve(__dirname,'../dist');
const heldAsset='static/img/journey/town-npcs-v1.png';
test('first launch waits for every asset, retries failure, and repairs missing cached art before play',async({browser})=>{
 test.setTimeout(65000);
 let hold=true,fail=true,waiting=false,release:()=>void=()=>{};
 let reachable=true;
 const server=createServer(async(req,res)=>{
  if(!reachable){req.socket.destroy();return;}
  const path=new URL(req.url!,'http://fixture').pathname.replace(/^\/play\//,'');res.setHeader('Cache-Control','no-store');
  if(path===heldAsset){if(hold)await new Promise<void>(resolve=>{release=resolve;waiting=true;});if(fail){res.statusCode=503;res.end('temporary failure');return;}}
  const file=!path||!path.includes('.')?'index.html':path;
  const mime=file.endsWith('.js')?'text/javascript':file.endsWith('.wasm')?'application/wasm':file.endsWith('.css')?'text/css':file.endsWith('.html')?'text/html':file.endsWith('.webmanifest')?'application/manifest+json':'application/octet-stream';res.setHeader('Content-Type',mime);
  try{res.end(readFileSync(join(built,file)));}catch{res.statusCode=404;res.end();}
 });
 await new Promise<void>(resolve=>server.listen(0,'127.0.0.1',resolve));const context=await browser.newContext({baseURL:`http://127.0.0.1:${(server.address() as any).port}/play/`});const page=await context.newPage();
 try{
  await page.goto('./',{waitUntil:'domcontentloaded'});await expect(page.locator('#launch-gate')).toBeVisible();await expect.poll(()=>waiting).toBe(true);await expect(page.locator('#main')).toHaveCount(0);await expect.poll(async()=>Number(await page.locator('#launch-progress').getAttribute('value'))).toBeGreaterThan(0);
  await page.screenshot({path:`test-results/first-download-${test.info().project.name}.png`});hold=false;release();await expect(page.locator('#launch-retry')).toBeVisible();await expect(page.locator('#main')).toHaveCount(0);await expect(page.locator('#launch-help')).toContainText('download could not finish');
  fail=false;await page.locator('#launch-retry').click();await expect(page.locator('#main')).toBeVisible({timeout:30000});await expect(page.locator('#launch-gate')).toHaveCount(0);
  const missing=await page.evaluate(async()=>{const m=await(await fetch('offline-manifest.json')).json();const cache=await caches.open('dystopian-trail:/play/:'+m.revision);return(await Promise.all(m.assets.map(async(a:any)=>await cache.match(new URL(a.path,document.baseURI))?null:a.path))).filter(Boolean);});expect(missing).toEqual([]);
  // A marker is insufficient: simulate one evicted image, then reload offline.
  // Headless Chromium can leave service-worker fetches online under setOffline.
  // Refuse fixture connections too, so the missing asset cannot be repaired.
  reachable=false;await context.setOffline(true);expect(await page.evaluate(()=>navigator.onLine)).toBe(false);
  const evicted=await page.evaluate(async path=>{let removed=false;for(const name of await caches.keys())if(name.startsWith('dystopian-trail:'))removed=await(await caches.open(name)).delete(new URL(path,document.baseURI))||removed;return removed;},heldAsset);expect(evicted).toBe(true);
  await page.reload({waitUntil:'domcontentloaded'});await expect(page.locator('#launch-retry')).toBeVisible({timeout:12000});await expect(page.locator('#main')).toHaveCount(0);
  hold=true;waiting=false;reachable=true;await context.setOffline(false);await page.locator('#launch-retry').click();await expect.poll(()=>waiting).toBe(true);await expect(page.locator('#main')).toHaveCount(0);hold=false;release();await expect(page.locator('#main')).toBeVisible();
  reachable=false;await context.setOffline(true);await page.reload();await expect(page.locator('#main')).toBeVisible();expect(await page.evaluate(async path=>(await(await fetch(path)).arrayBuffer()).byteLength,heldAsset)).toBe(readFileSync(join(built,heldAsset)).byteLength);
 }finally{release();await context.close();server.closeAllConnections();await new Promise<void>(resolve=>server.close(()=>resolve()));}
});

test('every image is decoded behind the gate and new encounter scenes use only prepared local art',async({page,context})=>{
 test.setTimeout(60000);
 await page.addInitScript(()=>{
  const decode=HTMLImageElement.prototype.decode;
  HTMLImageElement.prototype.decode=async function(){
   if(this.src.includes('encounter-settings-v2')) {
    (window as any).artWaiting=true;
    await new Promise<void>(resolve=>(window as any).releaseArt=resolve);
   }
   return decode.call(this);
  };
 });
 await page.goto('./',{waitUntil:'domcontentloaded'});
 await page.waitForFunction(()=>typeof (window as any).releaseArt==='function');
 await expect.poll(()=>page.evaluate(()=>(window as any).artWaiting)).toBe(true);
 await expect(page.locator('#main')).toHaveCount(0);await expect(page.locator('#launch-gate')).toBeVisible();
 await expect(page.locator('#launch-message')).toContainText('Preparing artwork');
 await page.screenshot({path:`test-results/art-preparation-${test.info().project.name}.png`});
 await page.evaluate(()=>(window as any).releaseArt());await waitForLaunch(page);await expect(page.locator('#main')).toBeVisible();
 const art=await page.evaluate(async()=>{
  const manifest=await(await fetch('offline-manifest.json')).json();
  const expected=manifest.assets.filter((a:any)=>/\.(png|jpe?g|webp|gif|svg|ico)$/i.test(a.path)).map((a:any)=>a.path).sort();
  return {expected,actual:Object.keys((window as any).dystrailAssetUrls).sort(),decoded:(window as any).dystrailPreparedImages.every((i:HTMLImageElement)=>i.complete&&i.naturalWidth>0)};
 });
 expect(art.actual).toEqual(art.expected);expect(art.decoded).toBe(true);
 const imageRequests:string[]=[];page.on('request',req=>{if(/^https?:/.test(req.url())&&/\.(png|jpe?g|svg|webp)(?:[?#]|$)/.test(req.url()))imageRequests.push(req.url());});
 await context.setOffline(true);
 await page.getByRole('button',{name:'Choose your character',exact:true}).click();
 await page.getByRole('radio',{name:'Journalist',exact:true}).click();await page.getByRole('button',{name:'Continue',exact:true}).click();
 await page.getByLabel('Your name',{exact:true}).fill('Offline Art');await page.getByLabel('Crew name',{exact:true}).fill('Prepared Crew');await page.getByRole('button',{name:'Continue',exact:true}).click();
 await page.getByRole('button',{name:'Review & depart',exact:true}).click();await page.getByRole('button',{name:'Start the journey',exact:true}).click();
 const gs=await savedState(page),events=JSON.parse(readFileSync('static/assets/data/game.json','utf8'));
 for(const id of ['classic_mutual_aid','sat_corn_bullets','raw_milk']) {
  const encounter=events.find((e:any)=>e.id===id);expect(encounter).toBeTruthy();
  gs.current_encounter=encounter;await importState(page,gs);
  await expect(page.locator('.scene-atlas image')).toHaveAttribute('href',/^blob:/);
  await page.evaluate(()=>new Promise<void>(resolve=>requestAnimationFrame(()=>resolve())));
  expect(await page.locator('.scene-speaker img').evaluateAll(es=>es.every(e=>(e as HTMLImageElement).complete&&(e as HTMLImageElement).naturalWidth>0))).toBe(true);
 }
 await snap(page,'prepared-encounter-art');expect(imageRequests).toEqual([]);
});
