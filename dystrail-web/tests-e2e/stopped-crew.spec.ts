import {test,expect,Page} from '@playwright/test';
import {baseline,importState,snap,fastMode} from './helpers';
import {atTown} from './geography';

const checkpoint=(page:Page)=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!));

async function expectParkedComposition(page:Page,count:number) {
 await expect(page.locator('.standing-member')).toHaveCount(count);
 const foreground=page.locator('.road-foreground');
 await expect(foreground).toBeVisible();
 const images=foreground.locator('img');
 expect(await images.count()).toBeGreaterThan(0);
 for(const image of await images.all()) {
  await expect(image).toHaveAttribute('src',/^blob:/);
  expect(await image.evaluate(async(image:HTMLImageElement)=>{await image.decode();return image.naturalWidth>0&&image.naturalHeight>0;})).toBe(true);
 }
 const layout=await page.locator('.journey-scene').evaluate(scene=>{
  const box=(element:Element)=>{const r=element.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,height:r.height};};
  const parked=scene.querySelector('.parked-crew')!,foreground=scene.querySelector('.road-foreground')!;
  const z=(element:Element)=>Number.parseInt(getComputedStyle(element).zIndex,10)||0;
  return {
   scene:box(scene),van:box(parked.querySelector('.crew-van')!),
   crew:[...parked.querySelectorAll('.standing-member')].map(element=>({...box(element),facing:new DOMMatrix(getComputedStyle(element).transform).a})).sort((a,b)=>a.left-b.left),
   foregroundAbove:z(foreground)>z(parked)||(z(foreground)===z(parked)&&Boolean(parked.compareDocumentPosition(foreground)&Node.DOCUMENT_POSITION_FOLLOWING)),
  };
 });
 const {scene,van,crew}=layout;
 const tolerance=1;
 expect(layout.foregroundAbove,'Foreground scenery must paint over the parked objects').toBe(true);
 for(const object of [van,...crew]) {
  expect(object.left).toBeGreaterThanOrEqual(scene.left-tolerance);
  expect(object.right).toBeLessThanOrEqual(scene.right+tolerance);
  expect(object.top).toBeGreaterThanOrEqual(scene.top-tolerance);
  expect(object.bottom).toBeLessThanOrEqual(scene.bottom+tolerance);
 }
 expect(van.bottom-scene.top,'The van should be pulled onto the lower shoulder').toBeGreaterThan(scene.height*.8);
 for(const member of crew) {
  expect(member.left,'Every person should stand to the right of the van').toBeGreaterThanOrEqual(van.right-tolerance);
  expect(Math.abs(van.bottom-member.bottom),'The crew and van should share the same shoulder').toBeLessThan(scene.height*.18);
 }
 const groupWidth=Math.max(...crew.map(c=>c.right))-Math.min(...crew.map(c=>c.left));
 expect(groupWidth,'The crew should form a compact discussion group').toBeLessThan((scene.right-scene.left)*.42);
 expect(crew.some(c=>c.facing>0)&&crew.some(c=>c.facing<0),'Both sides of the discussion should face inward').toBe(true);
 for(let a=0;a<crew.length;a++)for(let b=a+1;b<crew.length;b++) {
  const head=(c:typeof crew[number])=>({left:c.left+(c.right-c.left)*.24,right:c.right-(c.right-c.left)*.24,top:c.top+c.height*.08,bottom:c.top+c.height*.29});
  const one=head(crew[a]),two=head(crew[b]);
  const overlap=Math.max(0,Math.min(one.right,two.right)-Math.max(one.left,two.left))*Math.max(0,Math.min(one.bottom,two.bottom)-Math.max(one.top,two.top));
  expect(overlap,'Discussion faces should remain separately readable').toBeLessThan(2);
 }
 const byDepth=[...crew].sort((a,b)=>a.height-b.height);
 const rear=byDepth[0],front=byDepth.at(-1)!;
 expect(front.height-rear.height,'Front figures should look larger than rear figures').toBeGreaterThan(rear.height*.04);
 expect(front.bottom-rear.bottom,'Rear figures should stand higher in the scene').toBeGreaterThan(2);
}

test('paid shift returns to town without moving or spending time',async({page})=>{
 const gs=await baseline(page);atTown(gs,'Spokane');await importState(page,gs);
 await page.getByRole('button',{name:'Take a paid unloading shift',exact:true}).click();
 const exit=page.getByRole('button',{name:'Back to town',exact:true});
 await expect(exit).toHaveAttribute('id','outcome-continue');
 await expect(exit.locator('[data-icon=return]')).toHaveCount(1);
 await expect(exit.locator('[data-icon=travel]')).toHaveCount(0);
 const completed=await checkpoint(page);
 expect(completed.state.budget_cents).toBe(gs.budget_cents+1800);
 await snap(page,'paid-shift-back-to-town');
 await exit.focus();await page.keyboard.press('Enter');
 await expect(page.locator('#main')).toHaveAttribute('data-screen','town');
 const returned=await checkpoint(page);
 expect(returned.state.route_services).toEqual(completed.state.route_services);
 expect(returned.state.miles_traveled_actual).toBe(completed.state.miles_traveled_actual);
 expect(returned.state.clock_minutes).toBe(completed.state.clock_minutes);
 expect(returned.state.day).toBe(completed.state.day);
 expect(returned.travel_running).toBeFalsy();
 await expect(page.getByRole('button',{name:'Leave town',exact:true})).toBeEnabled();
 await expect(page.locator('.conditions-hud .leg-destination')).toContainText('Spokane');
 await page.waitForTimeout(1200);
 expect((await checkpoint(page)).state.miles_traveled_actual).toBe(completed.state.miles_traveled_actual);
});

test('stopping displays only present crew outside the van and keeps all readouts in the top HUD',async({page,context})=>{
 const gs=await baseline(page);
 await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene','open-pacific-northwest');
 await expectParkedComposition(page,6);
 await snap(page,'parked-full-crew');
 const california=structuredClone(gs);
 california.persona_id='whistleblower';california.route_services.route_id='whistleblower';
 await importState(page,california);
 await expect(page.locator('.journey-scene')).toHaveAttribute('data-scene','open-california-hills');
 await expectParkedComposition(page,6);
 await snap(page,'parked-california-crew');
 gs.party.members.find((m:any)=>m.persona==='satirist').status='Dead';
 gs.party.members.find((m:any)=>m.persona==='organizer').status='Departed';
 gs.encounter_cooldown=100;gs.crew_care.last_check_day=100;gs.weather_state.neutral_buffer=100;
 await importState(page,gs);
 await expectParkedComposition(page,4);
 await expect(page.locator('.van-occupant')).toHaveCount(0);
 await expect(page.locator('.standing-member[data-member=satirist],.standing-member[data-member=organizer]')).toHaveCount(0);
 await expect(page.locator('.scene-status,.leg-turn')).toHaveCount(0);
 for(const part of ['.leg-cash','.leg-vehicle','.leg-destination'])await expect(page.locator(`.conditions-hud ${part}`)).toBeVisible();
 const art=await page.locator('.standing-member image').first().getAttribute('href');expect(art).toMatch(/^blob:/);
 const readiness=await page.evaluate(async url=>{const i=new Image();i.src=url!;await i.decode();return [i.naturalWidth,i.naturalHeight];},art);
 expect(readiness).toEqual([1536,1024]);await snap(page,'stopped-crew-hud');
 await fastMode(page,true);await page.getByRole('button',{name:'Travel',exact:true}).click();
 await expect(page.locator('.standing-member')).toHaveCount(0);
 await expect(page.locator('.van-occupant')).toHaveCount(4);
 await page.getByRole('button',{name:'Pause travel',exact:true}).click();
 await expect(page.locator('.standing-member')).toHaveCount(4);await expect(page.locator('.van-occupant')).toHaveCount(0);
 await context.setOffline(true);await page.reload();
 await expectParkedComposition(page,4);await snap(page,'stopped-crew-offline');
});
