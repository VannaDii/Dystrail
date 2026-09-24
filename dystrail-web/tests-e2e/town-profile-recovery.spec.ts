import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,waitForLaunch} from './helpers';
const profiles=JSON.parse(readFileSync('static/assets/data/town-profiles.json','utf8'));
const routes=JSON.parse(readFileSync('../dystrail-game/data/routes.json','utf8'));
test('retained profiles match all 51 approved dated references',()=>{
 const units=JSON.parse(readFileSync('../review/recovery/current-source-records.json','utf8')).units.filter((u:any)=>u.category==='Town profiles');
 expect(units).toHaveLength(51);expect(profiles).toHaveLength(51);
 for(const unit of units){
  const p=profiles.find((p:any)=>p.town===unit.runtime_key);expect(p).toBeTruthy();
  expect(`${p.town}, ${p.state}. Population: ${p.population.toLocaleString('en-US')} (${p.population_year}). ${p.attraction.en}`).toBe(unit.source_paragraphs[1]);
 }
});
test('all town profiles show dated population, attraction and source links without changing the journey',async({page,context},info)=>{
 test.setTimeout(180000);const base=await baseline(page);
 for(const p of profiles){
  const route=routes.find((r:any)=>r.stops.some((s:any)=>s.name===p.town));const stop=route.stops.find((s:any)=>s.name===p.town);
  const s=structuredClone(base);s.seed=42;s.persona_id=route.id;
  s.route_services={...s.route_services,route_id:route.id,stop:stop.mile,trading:false,talked_at:null,talk_reward:null};
  s.activities.local_word=null;s.miles_traveled_actual=stop.mile/route.total_miles*s.trail_distance;s.miles_traveled=Math.round(s.miles_traveled_actual);
  await importState(page,s);
  const profile=page.locator('.town-profile');await expect(profile).toBeVisible();
  await expect(profile.locator('.town-vitals')).toContainText(p.state);await expect(profile.locator('.town-vitals')).toContainText(String(p.population_year));
  await expect(profile.locator('.town-vitals a')).toHaveAttribute('href',p.population_source);
  expect((await profile.locator('.town-vitals a').innerText()).replace(/\D/g,'')).toBe(String(p.population));
  await expect(profile.locator('.town-attraction p')).toHaveText(p.attraction.en);
  await expect(profile.locator('.town-attraction a')).toHaveAttribute('href',p.attraction_source);
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  const after=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')!).state);
  expect(after.stats).toEqual(s.stats);expect(after.party).toEqual(s.party);expect(after.clock_minutes).toBe(s.clock_minutes);
  if(p.town==='Minneapolis'){
   await context.setOffline(true);await page.reload();await waitForLaunch(page);await expect(profile.locator('.town-attraction p')).toHaveText(p.attraction.en);
   await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));await page.screenshot({path:info.outputPath('town-profile.png'),fullPage:true});await context.setOffline(false);
  }
 }
});
