import {test,expect,Page} from '@playwright/test';
import {openMenu,snap,waitForLaunch} from './helpers';
async function ready(page:Page){
 await page.goto('./');await waitForLaunch(page);await expect(page.locator('#main')).toBeVisible();
 await expect.poll(()=>page.evaluate(()=>(window as any).dystrailOffline?.state),{timeout:30000}).toBe('ready');
 await openMenu(page);
}
async function status(page:Page,state:string,percent=0){
 await page.evaluate(({state,percent})=>{(window as any).dystrailOffline={state,percent};window.dispatchEvent(new Event('dystrail-offline'));},{state,percent});
}
test('offline menu uses full width, centers help, and explains installation with a labeled action',async({page})=>{
 await ready(page);
 // Exercise the same guidance shown by browsers without a native install prompt.
 await page.evaluate(()=>{(window as any).dystrailInstallAvailable=false;window.dispatchEvent(new Event('dystrail-offline'));});
 const section=page.locator('.offline-status');await expect(section).toHaveAttribute('data-offline','ready');
 expect(await section.evaluate(e=>e.parentElement!.lastElementChild===e && e.lastElementChild!.classList.contains('offline-readiness'))).toBe(true);
 const width=await section.evaluate(e=>({actual:e.getBoundingClientRect().width,parent:e.parentElement!.getBoundingClientRect().width}));expect(Math.abs(width.actual-width.parent)).toBeLessThan(2);
 const help=section.getByRole('button',{name:'How this works: Offline play',exact:true});
 const geometry=await help.evaluate(e=>{const box=e.getBoundingClientRect();const range=document.createRange();range.selectNodeContents(e);const text=range.getBoundingClientRect();return {width:box.width,height:box.height,left:text.left-box.left,right:box.right-text.right,top:text.top-box.top,bottom:box.bottom-text.bottom};});
 expect(geometry.width).toBe(28);expect(geometry.height).toBe(28);expect(Math.min(geometry.left,geometry.right,geometry.top,geometry.bottom)).toBeGreaterThanOrEqual(0);
 await expect(section.getByRole('button',{name:'How to install',exact:true})).toBeVisible();await snap(page,'offline-menu-ready');
 await help.click();await expect(page.locator('.viewport-help')).toContainText('Ready for offline play');await page.locator('.wordmark').click();await expect(page.locator('.viewport-help')).toHaveCount(0);
 await openMenu(page);await page.getByRole('button',{name:'How to install',exact:true}).click();await expect(page.locator('.viewport-help')).toContainText('Add to Home Screen');
 const popup=await page.locator('.viewport-help').boundingBox();expect(popup!.x).toBeGreaterThanOrEqual(0);expect(popup!.x+popup!.width).toBeLessThanOrEqual(page.viewportSize()!.width);await snap(page,'offline-install-help');
 await page.locator('.wordmark').click();await openMenu(page);
 await status(page,'downloading',37);await expect(section).toContainText('37%');await expect(section.locator('progress')).toHaveAttribute('value','37');await snap(page,'offline-menu-downloading');
 await status(page,'error');await expect(section.locator('progress')).toHaveCount(0);await expect(section.getByRole('button',{name:'Retry download',exact:true})).toBeVisible();await snap(page,'offline-menu-error');
 await Promise.all([page.waitForEvent('domcontentloaded'),section.getByRole('button',{name:'Retry download',exact:true}).click()]);
 await waitForLaunch(page);await expect(page.locator('#main')).toBeVisible();await openMenu(page);await expect(section).toHaveAttribute('data-offline','ready');
 await page.locator('.language-picker>button').click();await page.getByRole('option',{name:/العربية/}).click();await expect(page.locator('html')).toHaveAttribute('dir','rtl');await expect(section).toContainText('جاهز للعب دون اتصال');await expect(section).toContainText('كيفية التثبيت');await snap(page,'offline-menu-arabic');
});

test('install availability updates while readiness stays unchanged and installed apps hide install controls',async({page})=>{
 await ready(page);const current=await page.evaluate(()=>(window as any).dystrailOffline);
 // The OS prompt is represented by a fixture: no app is installed on the test host.
 await page.evaluate(()=>{const event=new Event('beforeinstallprompt',{cancelable:true});(window as any).installPromptCalls=0;(event as any).prompt=async()=>{(window as any).installPromptCalls++;};window.dispatchEvent(event);});
 await expect(page.getByRole('button',{name:'Install Dystopian Trail',exact:true})).toBeVisible();await snap(page,'offline-menu-installable');
 await page.getByRole('button',{name:'Install Dystopian Trail',exact:true}).click();expect(await page.evaluate(()=>(window as any).installPromptCalls)).toBe(1);
 await expect(page.getByRole('button',{name:'How to install',exact:true})).toBeVisible();expect(await page.evaluate(()=>(window as any).dystrailOffline)).toEqual(current);
 await page.evaluate(()=>window.dispatchEvent(new Event('appinstalled')));await expect(page.locator('.pwa-install')).toHaveCount(0);await expect(page.locator('.offline-status')).toContainText('Ready for offline play');
 await snap(page,'offline-menu-installed');
});
