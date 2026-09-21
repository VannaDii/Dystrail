// Contact sheets lay out unchanged, actual scene captures for visual inspection.
import {readFileSync,writeFileSync,readdirSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import {chromium} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
const root='/private/tmp/dystrail-art-satire/review/art-satire/repair-ready-results';
const revision=JSON.parse(readFileSync('/private/tmp/dystrail-art-satire-build/offline-manifest.json')).revision;
const browser=await chromium.launch({headless:true,executablePath:'/Users/vanna/Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell'});
try {
 for(const device of ['chromium','mobile'])for(const state of ['offer','repaired']){
  const dir=`repair-presentation-repair-03538-r-without-duplicate-actions-${device}`;
  const files=readdirSync(`${root}/${dir}`).filter(p=>p.startsWith('REPAIR-')&&p.endsWith(`-${state}-scene.png`)).sort();
  if(files.length!==12)throw Error(`Missing ${device} ${state}: ${files.length}`);
  const html=`<!doctype html><meta charset="utf-8"><style>body{margin:0;padding:20px;background:#0b1c28;color:#f4eacb;font:16px Arial}h1{font-size:23px}main{display:grid;grid-template-columns:repeat(3,1fr);gap:12px}figure{margin:0;background:#152e3b}img{display:block;width:100%}figcaption{padding:8px}</style><h1>Repair scene review · ${device} · ${state} · ${revision}</h1><main>${files.map(f=>`<figure><img src="${dir}/${f}"><figcaption>${f.replace('-scene.png','')}</figcaption></figure>`).join('')}</main>`;
  const file=`${root}/${device}-${state}.html`;writeFileSync(file,html);
  const page=await browser.newPage({viewport:{width:1540,height:1000},deviceScaleFactor:1});
  await page.goto(pathToFileURL(file).href);await page.locator('img').evaluateAll(imgs=>Promise.all(imgs.map(i=>i.decode())));
  await page.screenshot({path:`${root}/${device}-${state}.png`,fullPage:true});await page.close();
 }
}finally{await browser.close();}
