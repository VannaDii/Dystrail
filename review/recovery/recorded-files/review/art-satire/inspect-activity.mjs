import {chromium} from './client-playwright.mjs';
const browser=await chromium.launch({headless:true});
const page=await browser.newPage();
await page.goto('http://127.0.0.1:62531/play/');
console.log(JSON.stringify(await page.locator('.scene-activity').evaluate(el=>({
 html:el.querySelector('.scene-atlas').outerHTML,
 boxes:[el,el.querySelector('.scene-art'),el.querySelector('svg'),el.querySelector('image')].map(e=>({tag:e.tagName,box:e.getBoundingClientRect().toJSON(),width:getComputedStyle(e).width,height:getComputedStyle(e).height,transform:getComputedStyle(e).transform})),
 view:el.querySelector('svg').viewBox.baseVal.toString(),scrollY:window.scrollY
})),null,2));
await browser.close();
