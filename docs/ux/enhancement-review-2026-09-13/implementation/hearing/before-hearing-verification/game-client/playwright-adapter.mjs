// Test-only bridge: run the supplied client against the native Yew hearing.
// No production testing hooks or additional game controls are introduced.
import {readFileSync} from 'node:fs';
import {chromium as installed} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
const checkpoint=readFileSync(new URL('./checkpoint.json',import.meta.url),'utf8');
export const chromium={launch:async options=>{
 const browser=await installed.launch({...options,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py'});
 const newPage=browser.newPage.bind(browser);
 browser.newPage=async options=>{
  const page=await newPage({...options,viewport:{width:1280,height:1000}});
  await page.addInitScript(checkpoint=>{
   localStorage.setItem('dystrail.autosave.v1',checkpoint);
   window.render_game_to_text=()=>JSON.stringify({
    coordinate_system:'DOM viewport: origin top-left, x rightward, y downward',
    screen:document.querySelector('#main')?.getAttribute('data-screen'),
    hearing_phase:document.querySelector('.hearing')?.getAttribute('data-hearing-phase'),
    visible_sanity:document.querySelector('[data-hearing-sanity]')?.textContent,
    narrative:document.querySelector('.hearing-dialogue')?.textContent,
    revealed_rounds:[...document.querySelectorAll('.hearing-rounds li')].map(el=>el.textContent),
    resolution:document.querySelector('.hearing-resolution')?.textContent,
    available_actions:[...document.querySelectorAll('.hearing-controls button')].map(el=>el.textContent)
   });
  },checkpoint);
  const goto=page.goto.bind(page);
  page.goto=async(...args)=>{const result=await goto(...args);await page.waitForFunction(()=>window.dystrailLaunch?.then);await page.evaluate(()=>window.dystrailLaunch);return result;};
  return page;
 };
 return browser;
}};
