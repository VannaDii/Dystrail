import {readFileSync,writeFileSync,readdirSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import {chromium} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
const root=process.argv[2],revision=process.argv[3];
const browser=await chromium.launch({headless:true,executablePath:'/Users/vanna/Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell'});
try{for(const device of ['chromium','mobile']){
 const dir=readdirSync(root).find(f=>f.startsWith('crossing-presentation-')&&f.endsWith(`-${device}`));if(!dir)throw Error(device);
 const files=readdirSync(`${root}/${dir}`).filter(f=>f.endsWith('-scene.png')).sort();
 for(let start=0;start<files.length;start+=6){
 const subset=files.slice(start,start+6),label=`${device}-${start/6+1}`;
 const html=`<!doctype html><meta charset="utf-8"><style>body{margin:0;padding:16px;background:#0b1c28;color:#f4eacb;font:16px Arial}h1{font-size:20px}main{display:grid;grid-template-columns:repeat(2,1fr);gap:12px}figure{margin:0}img{width:100%;display:block}figcaption{padding:8px}</style><h1>Crossing staging review · ${label} · ${revision}</h1><main>${subset.map(f=>`<figure><img src="${dir}/${f}"><figcaption>${f}</figcaption></figure>`).join('')}</main>`;
 const file=`${root}/${label}.html`;writeFileSync(file,html);const page=await browser.newPage({viewport:{width:1440,height:1000}});await page.goto(pathToFileURL(file).href);await page.locator('img').evaluateAll(xs=>Promise.all(xs.map(i=>i.decode())));await page.screenshot({path:`${root}/${label}.png`,fullPage:true});await page.close();
 }
}}finally{await browser.close();}
