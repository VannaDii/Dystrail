import {chromium as installed} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
import {readFileSync} from 'node:fs';
// The supplied game client remains unmodified. This adapter locates the installed
// browser and can restore an explicitly provided, disposable local-test checkpoint.
export const chromium={launch:async options=>{
 const browser=await installed.launch({...options,executablePath:'/Users/vanna/Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell'});
 if(process.env.DYSTRAIL_CLIENT_FIXTURE){
  const checkpoint=readFileSync(process.env.DYSTRAIL_CLIENT_FIXTURE,'utf8');
  const newPage=browser.newPage.bind(browser);
  browser.newPage=async options=>{
   const page=await newPage(options);
   await page.addInitScript(checkpoint=>{
    if(location.origin==='http://127.0.0.1:62531'&&!sessionStorage.getItem('art-client-fixture')){
     localStorage.setItem('dystrail.autosave.v1',checkpoint);
     localStorage.setItem('dystrail.locale','en');
     sessionStorage.setItem('art-client-fixture','1');
    }
   },checkpoint);
   const goto=page.goto.bind(page);
   page.goto=async(...args)=>{
    const response=await goto(...args);
    await page.waitForFunction(()=>typeof window.dystrailLaunch?.then==='function');
    await page.evaluate(()=>window.dystrailLaunch);
    return response;
   };
   return page;
  };
 }
 return browser;
}};
