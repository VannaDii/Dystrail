import {test, expect} from '@playwright/test';
import {baseline, waitForLaunch} from './helpers';

test('camp uses preserved symbol-free atlas online and offline', async ({page, context}, info) => {
  await page.setViewportSize({width:info.project.name === 'mobile' ? 390 : 1440,height:1000});
  await page.emulateMedia({reducedMotion:'reduce'});
  await baseline(page);
  await page.getByRole('button',{name:'Camp',exact:true}).click();
  const scene=page.locator('.scene-atlas[data-atlas="journey-settings-v1"][data-cell="3"]');
  await expect(scene).toBeVisible();
  const hash=await scene.locator('image').evaluate(async element=>{
    const url=element.getAttribute('href')!;
    const bytes=await (await fetch(url)).arrayBuffer();
    return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),n=>n.toString(16).padStart(2,'0')).join('');
  });
  expect(hash).toBe('813fedcd913da2356e6364561b217490fb550691bb73c0751bb3806b743f1de7');
  await page.evaluate(()=>{(document.activeElement as HTMLElement)?.blur();window.scrollTo(0,0);});
  await page.evaluate(()=>new Promise<void>(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve()))));
  await page.screenshot({path:info.outputPath('corrected-camp.png'),fullPage:true});
  await context.setOffline(true);
  await page.reload(); await waitForLaunch(page);
  // Camp is transient presentation; reopen it if restoring the saved travel state.
  if(!(await scene.count())) await page.getByRole('button',{name:'Camp',exact:true}).click();
  await expect(scene).toBeVisible();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});
