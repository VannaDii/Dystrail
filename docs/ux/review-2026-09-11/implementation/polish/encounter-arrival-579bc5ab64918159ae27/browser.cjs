const out='/tmp/dystrail-encounter-arrival-preview';
const origin=require('node:fs').readFileSync(out+'/origin.txt','utf8').trim();
const manifest=JSON.parse(require('node:fs').readFileSync(out+'/web/offline-manifest.json','utf8'));
const locales=Object.fromEntries(['en','it','es','ar','de','ta','te','ja','zh','ko','bn','hi','pa','mr','jv','id','pt','ru','fr','tr'].map(language=>[language,JSON.parse(require('node:fs').readFileSync(out+'/source/dystrail-web/i18n/'+language+'.json','utf8'))]));
const report={revision:manifest.revision,activation:[],layouts:[],errors:[]};
const fs=require('node:fs'), assert=require('node:assert/strict');
const {chromium}=require('/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright');
const {expect}=require('/Users/vanna/Source/Dystrail/dystrail-web/node_modules/@playwright/test');
const names={en:'English',it:'Italiano',es:'Español',ar:'العربية',zh:'中文',hi:'हिन्दी',fr:'Français',bn:'বাংলা',pt:'Português',ru:'Русский',ja:'日本語',de:'Deutsch',id:'Bahasa Indonesia',jv:'Basa Jawa',ko:'한국어',mr:'मराठी',pa:'ਪੰਜਾਬੀ',ta:'தமிழ்',te:'తెలుగు',tr:'Türkçe'};
const baselines=Object.fromEntries(['classic','deep'].map(mode=>[mode,JSON.parse(fs.readFileSync('/tmp/dystrail-crew-count-narrative/'+mode+'-baseline.json','utf8'))]));
async function menu(page){const button=page.locator('#game-menu-button');if(await button.getAttribute('aria-expanded')!=='true')await button.click()}
async function closeMenu(page){const button=page.locator('#game-menu-button');if(await button.getAttribute('aria-expanded')==='true')await button.click()}
async function importState(page,state,language='en'){await menu(page);await page.locator('#save-open-btn').click();if(await page.locator('.save-text-backup').getAttribute('open')===null)await page.locator('.save-text-backup>summary').click();await page.locator('#import-json').fill(JSON.stringify(state));await page.getByRole('button',{name:locales[language].save.import_button,exact:true}).click();await page.locator('.drawer').waitFor({state:'detached'});await closeMenu(page)}
async function chooseLanguage(page,language){await menu(page);await page.locator('.language-picker>button').click();await page.getByRole('option',{name:names[language],exact:true}).click();await closeMenu(page);await expect(page.locator('html')).toHaveAttribute('lang',language);await expect(page.locator('html')).toHaveAttribute('dir',language==='ar'?'rtl':'ltr')}
const checkpoint=page=>page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')).state);
async function freshPage(browser){const context=await browser.newContext({viewport:{width:1440,height:1000},reducedMotion:'reduce'}),page=await context.newPage();page.setDefaultTimeout(15000);page.on('pageerror',error=>report.errors.push({type:'pageerror',message:error.message}));page.on('console',message=>{if(message.type()==='error')report.errors.push({type:'console',message:message.text()})});await page.goto(origin);await page.locator('#main[data-screen=setup]').waitFor({timeout:90000});await page.waitForFunction(()=>window.dystrailOffline?.state==='ready');assert.equal(await page.evaluate(()=>window.dystrailOffline.revision),manifest.revision);return{context,page}}

async function openSave(page){await menu(page);await page.locator('#save-open-btn').click();await expect(page.locator('.save-manager')).toBeVisible()}
async function closeSave(page){await page.locator('.save-close').click();await expect(page.locator('.save-manager')).toHaveCount(0);await closeMenu(page)}
function canonical(state){const value=structuredClone(state);value.inventory.tags.sort();return value}
function sameState(actual,expected){assert.deepEqual(canonical(actual),canonical(expected))}
function persist(){fs.writeFileSync(out+'/browser-results.json',JSON.stringify(report,null,2))}

function approach(mode,clock){
 const s=structuredClone(baselines[mode]);
 s.route_services.route_id='uninterrupted-test-road';s.route_services.stop=null;
 s.crossings_completed=1;s.crossing_events=[];s.miles_traveled_actual=1220;s.miles_traveled=1220;s.prev_miles_traveled=1220;
 s.clock_minutes=clock;s.day_state.day_initialized=true;s.weather_state.today='Clear';s.weather_travel_multiplier=1;s.exec_travel_multiplier=1;s.exec_breakdown_bonus=0;
 s.illness_travel_penalty=1;s.vehicle.breakdown_cooldown=100;s.encounters.occurred_today=true;s.encounters_today=1;s.crew_care.last_check_day=100;s.last_encounter_driving_minutes=10000;
 s.journey_breakdown.base=0;s.journey_breakdown.beta=0;s.journey_wear.base=0;s.journey_wear.fatigue_k=0;
 s.journey_crossing.pass=1;s.journey_crossing.detour=0;s.journey_crossing.terminal=0;s.journey_crossing.bribe.pass_bonus=0;s.journey_crossing.bribe.detour_bonus=0;s.journey_crossing.bribe.terminal_penalty=0;
 s.inventory.tags=[];s.receipts=[];s.budget_cents=5000;s.budget=50;s.pace='steady';s.stats.hp=10;s.stats.sanity=10;
 return s;
}

const data=JSON.parse(fs.readFileSync(out+'/source/dystrail-web/static/assets/data/game.json','utf8'));
const convoy=data.find(e=>e.id==='deep_rustbelt_convoy');
let activePage;
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py'});
 try{
  for(const mode of ['classic','deep'])for(const [clock,detour] of [[480,false],[765,false]]){
   report.currentCase={mode,clock,detour};const {context,page}=await freshPage(browser);activePage=page;
   const s=approach(mode,clock);s.miles_traveled_actual=1249.5;s.miles_traveled=1249.5;s.prev_miles_traveled=1249.5;s.current_encounter=structuredClone(convoy);
   if(detour){s.journey_crossing.pass=0;s.journey_crossing.detour=1;s.journey_crossing.detour_hours={min:2,max:2}}
   await importState(page,s);const before=await checkpoint(page);
   await page.getByRole('button').filter({hasText:convoy.choices[0].label}).click();
   await page.waitForFunction(()=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')).state.current_encounter===null);
   const after=await checkpoint(page);
   await expect(page.locator('body')).toContainText(locales.en.play_log.bribe);
   await expect(page.locator('body')).toContainText(locales.en.play_log.crossed);
   assert.equal(after.miles_traveled_actual,1250);assert.equal(after.crossings_completed,2);assert.equal(after.crossing_events.length,1);
   assert.equal(after.driving_minutes_total-before.driving_minutes_total,1);
   const elapsed=(after.day-before.day)*300+after.clock_minutes-before.clock_minutes;
   assert.equal(elapsed,detour?150:60);
   const bribeCents=mode==='classic'?1000:1500;assert.equal(after.crossing_events[0].bribe_cost_cents,bribeCents);
   if(after.day>before.day)assert.equal(after.day_state.day_initialized,true);
   await page.waitForTimeout(500);sameState(await checkpoint(page),after);
   await page.screenshot({path:out+'/'+mode+'-'+clock+'-'+(detour?'detour':'pass')+'.png',fullPage:true});
   await context.setOffline(true);await page.reload();await page.waitForFunction(()=>window.dystrailOffline?.state==='ready');sameState(await checkpoint(page),after);
   report.activation.push({mode,startClock:clock,outcome:detour?'detour':'pass',exactCrossing:1250,drivingMinutes:1,elapsedMinutes:elapsed,bribeCents,crossingResolvedOnce:true,finalDay:after.day,finalClock:after.clock_minutes,pauseAndOfflineReloadUnchanged:true});persist();
   await context.close();
  }
  assert.deepEqual(report.errors,[]);persist();console.log(JSON.stringify(report));
 }catch(error){if(activePage&&!activePage.isClosed()){await activePage.screenshot({path:out+'/failure.png',fullPage:true});fs.writeFileSync(out+'/failure-body.txt',await activePage.locator('body').innerText());fs.writeFileSync(out+'/failure-state.json',JSON.stringify(await checkpoint(activePage),null,2))}throw error}finally{await browser.close()}
})().catch(error=>{report.failure=error.stack;persist();console.error(error);process.exit(1)});
