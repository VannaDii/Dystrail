const out='/tmp/dystrail-mileage-ledger-preview';
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


function fractionalRoad(mode){const s=structuredClone(baselines[mode]);s.route_services.route_id='uninterrupted-test-road';s.day_state.day_initialized=true;s.weather_state.today='Clear';s.weather_state.neutral_buffer=100;s.weather_travel_multiplier=0.917;s.exec_travel_multiplier=1;s.exec_breakdown_bonus=0;s.illness_travel_penalty=1;s.vehicle.breakdown_cooldown=100;s.last_encounter_driving_minutes=10000;s.crew_care.last_check_day=100;s.current_order=null;s.exec_order_days_remaining=0;s.exec_order_cooldown=100;s.disease_cooldown=100;s.stats.hp=10;s.stats.sanity=8;s.stats.supplies=16;s.pace='heated';s.driving_minutes_total=0;s.pace_fatigue_remainder=0;s.current_day_record={day_index:s.day-1,kind:'non_travel',miles:0,tags:[]};s.current_day_kind=null;s.current_day_miles=0;s.current_day_reason_tags=[];return s}
async function fastMode(page){const b=page.getByRole('switch',{name:'Fast mode',exact:true});if(await b.getAttribute('aria-checked')!=='true')await b.click()}
async function driveHour(page){const before=await checkpoint(page);await page.getByRole('button',{name:'Travel',exact:true}).click();await page.waitForFunction(n=>JSON.parse(localStorage.getItem('dystrail.autosave.v1')).state.driving_minutes_total>=n+60,before.driving_minutes_total);await page.getByRole('button',{name:'Pause travel',exact:true}).click();await expect(page.locator('#main')).toHaveAttribute('data-screen','travel');const after=await checkpoint(page);assert.equal(after.driving_minutes_total-before.driving_minutes_total,60);return after}
function conserved(s){const records=[...s.day_records,...(s.current_day_record?[s.current_day_record]:[])];const total=Math.fround(records.reduce((sum,r)=>sum+Math.fround(r.miles),0));assert.equal(total,Math.fround(s.miles_traveled_actual));if(s.day_state.day_initialized){assert.equal(Math.fround(s.current_day_miles),Math.fround(Math.fround(s.miles_traveled_actual)-Math.fround(s.prev_miles_traveled)+Math.fround(s.distance_remainder)-Math.fround(s.day_start_remainder)))}return{miles:s.miles_traveled_actual,ledgerTotal:total,completedDays:s.day_records.length,openMiles:s.current_day_record?.miles??0}}
let activePage;
(async()=>{const browser=await chromium.launch({headless:true,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py'});try{
for(const mode of ['classic','deep'])for(const width of [1440,393]){
 report.currentCase={mode,width};const{context,page}=await freshPage(browser);activePage=page;await page.setViewportSize({width,height:width===393?852:1000});await importState(page,fractionalRoad(mode));await fastMode(page);const hours=[];
 for(let h=1;h<=6;h++){
  const after=await driveHour(page);hours.push({hour:h,day:after.day,clock:after.clock_minutes,...conserved(after)});
  if(h===2||h===5){await context.setOffline(true);await page.reload();await page.waitForFunction(()=>window.dystrailOffline?.state==='ready');sameState(await checkpoint(page),after);conserved(await checkpoint(page));await fastMode(page)}
 }
 const after=await checkpoint(page);assert.equal(after.driving_minutes_total,360);assert.equal(after.day_records.length,1);assert(after.current_day_record.miles>0);assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);await page.screenshot({path:out+`/ledger-${mode}-${width}.png`,fullPage:true});
 report.activation.push({mode,width,hours,observedDistanceEqualsDayLedger:true,offlineReloadAfterHours:[2,5],noHorizontalOverflow:true});persist();await context.close();
}
assert.deepEqual(report.errors,[]);delete report.currentCase;persist();console.log(JSON.stringify({revision:manifest.revision,cases:report.activation.length,drivingHours:24,errors:report.errors}));
}catch(error){if(activePage&&!activePage.isClosed()){await activePage.screenshot({path:out+'/failure.png',fullPage:true});fs.writeFileSync(out+'/failure-body.txt',await activePage.locator('body').innerText());fs.writeFileSync(out+'/failure-state.json',JSON.stringify(await checkpoint(activePage),null,2))}throw error}finally{await browser.close()}})().catch(error=>{report.failure=error.stack;persist();console.error(error);process.exit(1)});
