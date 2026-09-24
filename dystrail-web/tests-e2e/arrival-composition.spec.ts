import {test,expect} from '@playwright/test';
import {baseline,importState,waitForLaunch} from './helpers';
test('arrival keeps a shared plaza camera and only actual surviving travelers',async({page,context},info)=>{
 const s=await baseline(page);s.seed=42;
 s.boss={ready:true,reached:true,attempted:false,victory:false,presentation:'Arrival',hearing:null};
 s.current_encounter=null;s.route_services.stop=null;s.miles_traveled_actual=s.trail_distance;s.miles_traveled=s.trail_distance;
 for(const missing of [false,true]){
  if(missing){s.party.members[0].status='Departed';s.party.members[1].status='Dead';}
  await importState(page,s);const scene=page.locator('.hearing-arrival .journey-scene');
  await expect(scene.locator('.crew-van')).toHaveCount(0);
  await expect(scene.locator('.standing-member')).toHaveCount(missing?4:6);
  for(const member of s.party.members)await expect(scene.locator(`.standing-member[data-member="${member.persona}"]`)).toHaveCount(member.status==='Active'?1:0);
  const rect=await scene.boundingBox();expect(rect!.width/rect!.height).toBeCloseTo(1.5,2);
  await context.setOffline(true);await page.reload();await waitForLaunch(page);
  await expect(scene.locator('.standing-member')).toHaveCount(missing?4:6);
  expect(await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state.party)).toEqual(s.party);
  await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await scene.screenshot({path:info.outputPath(missing?'arrival-survivors.png':'arrival-full-crew.png'),animations:'disabled'});await context.setOffline(false);
 }
});
