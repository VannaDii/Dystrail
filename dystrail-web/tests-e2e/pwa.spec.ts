import {test,expect,chromium} from '@playwright/test';
import {readFileSync,mkdtempSync,rmSync} from 'node:fs';
import {join,resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {createHash} from 'node:crypto';
import {createServer} from 'node:http';
import {baseline,importState,savedState,snap,openMenu} from './helpers';
import {atTown} from './geography';
const built=process.env.PLAYTEST_DIST || resolve(__dirname,'../dist');

test('all assets, saved play and town scenes work after an offline relaunch',async({baseURL,launchOptions})=>{
 test.setTimeout(60000);
 const profile=mkdtempSync(join(tmpdir(),'dystrail-install-'));
 const context=await chromium.launchPersistentContext(profile,{...launchOptions,channel:process.env.PLAYTEST_CHANNEL,headless:true,baseURL});
 const page=await context.newPage();
 try {
 const gs=await baseline(page);
 await expect.poll(()=>page.evaluate(()=>(window as any).dystrailOffline?.state),{timeout:30000}).toBe('ready');
 const manifest=await page.evaluate(async()=>await (await fetch('offline-manifest.json')).json());
 const appManifest=await page.evaluate(async()=>await (await fetch('static/manifest.webmanifest')).json());
 expect(appManifest.name).toBe('Dystopian Trail');expect(appManifest.display).toBe('standalone');expect(appManifest.icons.map((i:any)=>i.sizes)).toEqual(['192x192','512x512']);
 const cdp=await context.newCDPSession(page);const installability=await cdp.send('Page.getInstallabilityErrors');expect(installability.installabilityErrors).toEqual([]);
 await context.setOffline(true);await page.close();const offline=await context.newPage();await offline.goto('./travel');
 await expect(offline.locator('#main')).toHaveAttribute('data-screen','travel');expect((await savedState(offline)).party).toEqual(gs.party);
 const failed=await offline.evaluate(async(assets:any[])=>{
   const failures=[];for(const asset of assets){try{const r=await fetch(new URL(asset.path,document.baseURI));if(!r.ok||(await r.arrayBuffer()).byteLength!==asset.bytes)failures.push(asset.path);}catch{failures.push(asset.path);}}return failures;
 },manifest.assets);expect(failed).toEqual([]);
 atTown(gs,'Madison');await importState(offline,gs);await offline.getByRole('button',{name:/^Talk to locals/}).click();await expect(offline.locator('.local-fact')).toContainText('UW–Madison');await snap(offline,'offline-town');
 await openMenu(offline);await expect(offline.locator('.offline-status')).toContainText('Ready for offline play');
 } finally {await context.close();rmSync(profile,{recursive:true,force:true});}
});

test('a complete remote update loads first and an interrupted update falls back without losing saves',async({browser})=>{
 test.setTimeout(90000);
 const original=JSON.parse(readFileSync(join(built,'offline-manifest.json'),'utf8'));
 const worker=readFileSync('../scripts/offline-worker.js','utf8');
 const html=readFileSync(join(built,'index.html'),'utf8');
 let version='one';let fail=false;let hold=false;let release:()=>void=()=>{};let waiting=false;
 const updateHtml=()=>html.replace('</head>',`<meta name="test-build" content="${version}"></head>`);
 const updateBuild=()=>({...original,revision:original.revision+'-'+version,assets:original.assets.map((a:any)=>a.path==='index.html'?{...a,bytes:Buffer.byteLength(updateHtml()),integrity:'sha256-'+createHash('sha256').update(updateHtml()).digest('base64')}:a)});
 const server=createServer(async(req,res)=>{
  res.setHeader('Cache-Control','no-store');
  const path=new URL(req.url!,'http://local').pathname.replace(/^\/play\//,'');
  if(path==='sw.js'){res.setHeader('Content-Type','text/javascript');res.end('const BUILD='+JSON.stringify(updateBuild())+';\n'+worker);return;}
  if(path==='static/img/journey/town-npcs-v1.png'&&version!=='one'){
   if(fail){res.statusCode=503;res.end('update interrupted');return;}
   if(hold)await new Promise<void>(resolve=>{release=resolve;waiting=true;});
  }
  const mime=path.endsWith('.js')?'text/javascript':path.endsWith('.wasm')?'application/wasm':path.endsWith('.css')?'text/css':path.endsWith('.webmanifest')?'application/manifest+json':undefined;
  if(mime)res.setHeader('Content-Type',mime);
  if(path==='index.html'||!path||!path.includes('.')){res.setHeader('Content-Type','text/html');res.end(updateHtml());return;}
  try{res.end(readFileSync(join(built,path)));}catch{res.statusCode=404;res.end();}
 });
 await new Promise<void>(resolve=>server.listen(0,'127.0.0.1',resolve));const port=(server.address() as any).port;
 const context=await browser.newContext({baseURL:`http://127.0.0.1:${port}/play/`});const page=await context.newPage();
 try{
  const gs=await baseline(page);await expect.poll(()=>page.evaluate(()=>(window as any).dystrailOffline?.state),{timeout:30000}).toBe('ready');
  version='two';hold=true;await page.reload({waitUntil:'domcontentloaded'});
  await expect(page.locator('#launch-gate')).toBeVisible();await expect(page.locator('#main')).toHaveCount(0);
  await expect.poll(()=>waiting,{timeout:10000}).toBe(true);hold=false;release();
  await expect(page.locator('meta[name=test-build]')).toHaveAttribute('content','two',{timeout:30000});await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');expect((await savedState(page)).party).toEqual(gs.party);
  version='three';fail=true;await page.reload();await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');await expect(page.locator('meta[name=test-build]')).toHaveAttribute('content','two');expect((await savedState(page)).party).toEqual(gs.party);
  await context.setOffline(true);await page.reload();await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');expect((await savedState(page)).party).toEqual(gs.party);
 } finally{release();await context.close();server.closeAllConnections();await new Promise<void>(resolve=>server.close(()=>resolve()));}
});
