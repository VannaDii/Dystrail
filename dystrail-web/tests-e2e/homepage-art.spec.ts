import {test,expect} from '@playwright/test';
import {baseline,importState,waitForLaunch} from './helpers';
import {resolve} from 'node:path';

test('capture approved current cast for existing homepage artwork',async({page})=>{
 test.skip(test.info().project.name==='mobile','One export produces both responsive sizes.');
 test.setTimeout(90000);
 await page.setViewportSize({width:1440,height:1200});
 await page.emulateMedia({reducedMotion:'reduce'});
 const s=await baseline(page);s.seed=42;s.clock_minutes=720;s.turn_journal_start=null;
 await importState(page,s);await page.reload();await waitForLaunch(page);
 await page.locator('.world-view').screenshot({path:resolve('../site/assets/gameplay-current.jpg'),type:'jpeg',quality:88,animations:'disabled'});
 // A recorded crossing keeps all six seated without advancing the travel clock.
 s.crossing_events=[{day:s.day,region:s.region,season:s.season,kind:'checkpoint',permit_used:false,bribe_attempted:false,bribe_success:null,bribe_cost_cents:0,bribe_chance:null,bribe_roll:null,detour_reason:null,detour_taken:false,detour_hours:null,detour_base_supplies_delta:null,detour_extra_supplies_loss:null,terminal_threshold:0,terminal_roll:null,outcome:'passed'}];
 s.visual_content={edition:1,selections:{},outcomes:{},policy_bulletins:[],crossing_presentations:[{event_index:0,unit:'CROSS-02C-B',permit_receipt:false,acknowledged:false}]};
 await importState(page,s);await expect(page.locator('.van-occupant')).toHaveCount(6);
 // Capture the existing text-free scene for the fixed 1200 x 630 social surface.
 const scene=await page.locator('.scene-art').boundingBox();
 await page.screenshot({path:resolve('../site/assets/gameplay-social.jpg'),type:'jpeg',quality:88,clip:{x:scene!.x+(scene!.width-1200)/2,y:scene!.y+(scene!.height-630)/2,width:1200,height:630},animations:'disabled'});
 // Isolate the actual rendered vehicle component, keeping its existing masks,
 // seats and sprites. This is a DOM capture, not a newly painted cast.
 await page.evaluate(()=>{const van=document.querySelector('.crew-van')!.cloneNode(true);document.body.replaceChildren(van);});
 await page.addStyleTag({content:'html,body{margin:0!important;padding:0!important;background:transparent!important;min-width:0!important;overflow:hidden!important}body>.crew-van{position:fixed!important;inset:0!important;width:100vw!important;height:100vh!important;aspect-ratio:auto!important;filter:none!important;animation:none!important}'});
 for(const [name,width,height] of [['crew-van-current',1536,1024],['crew-van-current-small',768,512]] as const){
  await page.setViewportSize({width,height});
  await expect(page.locator('.van-occupant')).toHaveCount(6);
  await page.screenshot({path:resolve(`../site/assets/${name}.png`),omitBackground:true,animations:'disabled'});
 }
 await page.locator('.van-seating').evaluate(el=>el.remove());
 await page.setViewportSize({width:384,height:256});
 await page.screenshot({path:resolve('static/img/status/vehicle-v2.png'),omitBackground:true,animations:'disabled'});
});

test('homepage keeps its layout while showing current artwork',async({page},info)=>{
 await page.setViewportSize({width:info.project.name==='mobile'?390:1440,height:1000});
 await page.emulateMedia({reducedMotion:'reduce'});
 await page.goto('http://127.0.0.1:62528/');
 await expect(page.locator('.hero-van img')).toBeVisible();
 const images=await page.locator('img').evaluateAll(async images=>{await Promise.all(images.map(i=>i.decode()));return images.map(i=>({src:i.currentSrc,width:i.naturalWidth}));});
 expect(images.every(i=>i.width>0)).toBe(true);
 expect(images.find(i=>i.src.includes('crew-van-current'))).toBeTruthy();
 expect(images.some(i=>/crew-van(?:-small)?\.webp/.test(i.src))).toBe(false);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 await expect(page.locator('.hero .play-button')).toHaveAttribute('href','/play/');
 const social=await page.evaluate(async()=>{const url=new URL(document.querySelector<HTMLMetaElement>('meta[property="og:image"]')!.content);const i=new Image();i.src=url.pathname;await i.decode();return [i.naturalWidth,i.naturalHeight];});
 expect(social).toEqual([1200,630]);
 await page.screenshot({path:info.outputPath('homepage-current.png'),fullPage:true});
});

test('vehicle summary uses the current unoccupied vehicle icon offline',async({page,context},info)=>{
 const s=await baseline(page);s.seed=42;
 await importState(page,s);
 await context.setOffline(true);await page.reload();await waitForLaunch(page);
 await page.getByRole('tab',{name:'The Van',exact:true}).click();
 const icons=await page.locator('.van-overview img').evaluateAll(async images=>{await Promise.all(images.map(i=>(i as HTMLImageElement).decode()));return images.map(i=>decodeURIComponent(i.getAttribute('src')!));});
 expect(icons.some(src=>src.includes('status/vehicle-v2.png'))).toBe(true);
 expect(icons.some(src=>src.includes('journey/van-crew.png'))).toBe(false);
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
 await page.screenshot({path:info.outputPath('vehicle-summary-current.png'),fullPage:true});
});
