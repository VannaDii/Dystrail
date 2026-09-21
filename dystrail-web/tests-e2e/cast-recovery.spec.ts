import {test, expect} from '@playwright/test';
import {baseline, importState, waitForLaunch} from './helpers';

test('retained cast follows surviving crew and driver reassignment after import and offline reload', async ({page, context}, info) => {
  await page.setViewportSize({width:info.project.name === 'mobile' ? 390 : 1440,height:1000});
  await page.emulateMedia({reducedMotion:'reduce'});
  const state=await baseline(page);
  state.party.members.find((m:any)=>m.persona==='staffer').status='Departed';
  state.party.members.find((m:any)=>m.persona==='organizer').status='Dead';
  await importState(page,state);
  await expect(page.locator('.standing-member')).toHaveCount(4);
  await expect(page.locator('.standing-member[data-member="staffer"],.standing-member[data-member="organizer"]')).toHaveCount(0);
  await expect(page.locator('.scene-footer .scene-speaker')).toHaveCount(0);
  const geometry=await page.locator('.scene-art').evaluate(art=>{
    const scene=art.getBoundingClientRect(),van=art.querySelector('.crew-van')!.getBoundingClientRect();
    return {sceneRatio:scene.width/scene.height,vanRatio:van.width/van.height,vanShare:van.width/scene.width};
  });
  expect(geometry.sceneRatio).toBeCloseTo(1.5,2);
  expect(geometry.vanRatio).toBeCloseTo(1.5,2);
  expect(geometry.vanShare).toBeCloseTo(.38,2);
  await page.getByRole('button',{name:'Travel',exact:true}).click();
  await expect(page.locator('.van-occupant')).toHaveCount(4);
  await expect(page.locator('.van-occupant[data-seat="4"]')).toHaveAttribute('data-member','satirist');
  await expect(page.locator('.van-occupant[data-seat="4"]')).toHaveAttribute('data-pose','7');
  await page.getByRole('button',{name:'Pause travel',exact:true}).click();
  await context.setOffline(true);
  await page.reload();await waitForLaunch(page);
  await expect(page.locator('.standing-member')).toHaveCount(4);
  const castReady=await page.evaluate(()=>Object.keys((window as any).dystrailAssetUrls).filter(p=>p.startsWith('static/img/cast-v2/')).length);
  expect(castReady).toBe(6);
  await page.evaluate(()=>window.scrollTo(0,0));
  await page.screenshot({path:info.outputPath('surviving-crew-offline.png'),fullPage:true});
});
