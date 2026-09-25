// Lay out unmodified browser captures in HTML, then screenshot that document.
// No image pixels are edited or regenerated.
import {readFileSync,writeFileSync} from 'node:fs';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {dirname,join} from 'node:path';
import {chromium} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
const root=join(dirname(fileURLToPath(import.meta.url)),'current-previews');
const data=JSON.parse(readFileSync(join(root,'manifest.json'),'utf8'));
const escape=s=>String(s).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const browser=await chromium.launch({headless:true});
const outputs=[];
try {
 for(const device of ['desktop','mobile']){
  const width=device==='desktop'?1520:1280;
  const cards=data.frames.map((f,i)=>`<article class="${f.status==='Partial'?'partial':''}"><div class="card-head"><span class="number">${String(i+1).padStart(2,'0')}</span><h2>${escape(f.title)}</h2><b>${escape(f.status)}</b></div><p>${escape(f.note)}</p><a href="${device}/${f.id}/shot-0.png"><img alt="${escape(f.title)}: actual ${device} game capture" src="${device}/${f.id}/shot-0.png"></a></article>`).join('');
  const html=`<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Dystrail · ${device} preview sheet</title><style>
  *{box-sizing:border-box}body{margin:0;background:#081720;color:#f6edcf;font-family:Arial,sans-serif;padding:36px}header{border-top:5px solid #f1cf6b;padding-top:22px;margin-bottom:28px}small{font-size:15px;letter-spacing:2px;text-transform:uppercase;color:#f1cf6b}h1{font-size:40px;margin:10px 0 14px}header p{font-size:18px;line-height:1.5;max-width:1200px;margin:8px 0;color:#c1d0d5}.grid{display:grid;grid-template-columns:repeat(${device==='desktop'?2:3},minmax(0,1fr));gap:24px;align-items:start}article{border:1px solid #48616d;border-radius:8px;overflow:hidden;background:#112936}article.partial{border-color:#d3a359}.card-head{display:flex;align-items:center;gap:10px;padding:18px 16px 0}.number{color:#91b9c0;font-size:14px}h2{font-size:19px;line-height:1.25;margin:0;flex:1}b{color:#acd6c5;background:#213f3f;font-size:12px;padding:6px 8px;border-radius:4px;white-space:nowrap}.partial b{color:#ffe0a0;background:#4b3a23}article p{font-size:15px;line-height:1.5;color:#c1d0d5;padding:0 16px;min-height:68px}a{display:block}img{display:block;width:100%;height:auto}footer{border-top:1px solid #48616d;margin-top:30px;padding-top:22px;font-size:17px;line-height:1.5;color:#c1d0d5}footer strong{color:#f1cf6b}
  @media(max-width:800px){body{padding:16px}.grid{grid-template-columns:1fr}h1{font-size:30px}}
  </style><header><small>Dystrail · visual world and satire integration</small><h1>${device==='desktop'?'Desktop':'Mobile'} preview sheet</h1><p>Actual game captures from candidate <strong>${escape(data.revision)}</strong> · ${device==='desktop'?'1440 px desktop viewport':'390 px mobile viewport'} · September 25, 2026.</p><p><strong>Work in progress.</strong> “Integrated” identifies the illustrated feature, not completion of the whole game. Click any screen to inspect its full-size capture.</p></header><main class="grid">${cards}</main><footer><strong>Still required:</strong> finish remaining retained-art integration and localized content, resolve explicit artwork review items, and validate the combined release across saves, replay, offline operation and accessibility. The 36 mechanics-dependent source records remain inactive.<br>Playtest build 0fd8b1d is live; these sheets capture candidate ${escape(data.revision)} and are not final release acceptance.</footer></html>`;
  const file=join(root,`${device}.html`);writeFileSync(file,html);
  const page=await browser.newPage({viewport:{width,height:1000},deviceScaleFactor:1});
  await page.goto(pathToFileURL(file).href);await page.locator('img').evaluateAll(images=>Promise.all(images.map(img=>img.decode())));
  const stats=await page.evaluate(()=>({width:document.documentElement.scrollWidth,height:document.documentElement.scrollHeight,images:[...document.images].map(i=>({src:i.getAttribute('src'),width:i.naturalWidth,height:i.naturalHeight,complete:i.complete}))}));
  if(stats.width>width||stats.images.length!==data.frames.length||stats.images.some(i=>!i.complete||!i.width))throw new Error('Incomplete sheet rendering');
  await page.screenshot({path:join(root,`${device}-preview-sheet.png`),fullPage:true});
  outputs.push({device,...stats});await page.close();
 }
 writeFileSync(join(root,'sheet-render.json'),JSON.stringify({revision:data.revision,outputs},null,2)+'\n');
 console.log(JSON.stringify(outputs.map(({device,width,height})=>({device,width,height}))));
} finally {await browser.close();}
