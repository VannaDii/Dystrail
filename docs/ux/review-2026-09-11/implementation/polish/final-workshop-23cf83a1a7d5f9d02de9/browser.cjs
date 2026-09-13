const out='/tmp/dystrail-final-workshop-build';
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


const data=JSON.parse(fs.readFileSync(out+'/source/dystrail-web/static/assets/data/game.json','utf8'));
const tariff=data.find(e=>e.id==='sat_alternator_tariff');
function ready(mode){const s=structuredClone(baselines[mode]);s.day=6;s.clock_minutes=480;s.day_state.day_initialized=true;s.budget_cents=5000;s.budget=50;s.stats={...s.stats,supplies:10,hp:10,sanity:8,morale:8,allies:3};s.current_encounter=null;s.ending=null;s.abandoned=false;s.ally_notice=null;s.breakdown=null;s.boss.ready=false;s.crew_care={reason:0,strain:{},pending:null,last_check_day:6};s.activities.local_word=null;s.route_services.stop=null;s.route_services.map_reviewed=null;s.journal=[];s.turn_journal_start=0;return s}
function encounterCopy(lang){if(lang==='en')return{name:tariff.name,desc:tariff.desc,...Object.fromEntries(tariff.choices.flatMap((c,i)=>[[`choice_${i}`,c.label],[`log_${i}`,c.effects.log]]))};return locales[lang].encounter_copy[tariff.id]}
async function layout(page){return page.evaluate(()=>({viewport:innerWidth,scrollWidth:document.documentElement.scrollWidth,overflowingButtons:[...document.querySelectorAll('.encounter-choice button,.crew-incident .camp-actions button,#outcome-continue')].filter(b=>b.getClientRects().length).map(b=>({text:b.textContent.trim(),rect:b.getBoundingClientRect().toJSON()})).filter(b=>b.rect.left < -1 || b.rect.right>innerWidth+1)}))}
let activePage;
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py'});
 try{
  for(const mode of ['classic','deep'])for(const language of ['en','es','it','ar']){
   const {context,page}=await freshPage(browser);activePage=page;await importState(page,ready(mode));if(language!=='en')await chooseLanguage(page,language);
   const copy=encounterCopy(language);
   for(const width of [1440,393]){
    report.currentCase={mode,language,width,kind:'tariff'};await page.setViewportSize({width,height:width===393?852:1000});
    const s=ready(mode);s.current_encounter=structuredClone(tariff);await importState(page,s,language);
    await expect(page.locator('.journey-scene h1')).toHaveText(copy.name);
    await expect(page.locator('.encounter-desc p')).toHaveText(copy.desc);
    const buttons=page.locator('.encounter-choice button');await expect(buttons).toHaveCount(3);
    for(let i=0;i<3;i++)await expect(buttons.nth(i).locator('.action-title')).toHaveText(copy[`choice_${i}`]);
    const dimensions=await layout(page);assert(dimensions.scrollWidth<=width+1);assert.deepEqual(dimensions.overflowingButtons,[]);
    if(language==='en'||(language==='ar'&&width===393))await page.screenshot({path:out+`/tariff-${mode}-${language}-${width}.png`,fullPage:true});
    const before=await checkpoint(page);await buttons.nth(1).click();await expect(page.locator('.outcome-summary')).toContainText(copy.log_1);
    const after=await checkpoint(page);assert.equal(after.budget_cents,before.budget_cents);assert.equal(after.stats.credibility,before.stats.credibility+2);assert.equal(after.stats.sanity,before.stats.sanity-1);assert.equal(after.clock_minutes,before.clock_minutes+30);assert.equal(after.receipts.length,before.receipts.length+1);
    const outcomeLayout=await layout(page);assert(outcomeLayout.scrollWidth<=width+1);assert.deepEqual(outcomeLayout.overflowingButtons,[]);
    report.layouts.push({...report.currentCase,exactTitleDescriptionChoicesAndOutcome:true,quoteCashUnchanged:true,receiptAdded:true,choiceMinutes:30,noHorizontalOverflow:true});persist();
   }
   report.currentCase={mode,language,width:393,kind:'named-care'};
   const s=ready(mode),member=s.party.members.find(m=>m.persona==='organizer');assert(member);member.name='Mika Chen';member.status='Active';s.crew_care={reason:0,strain:{[member.persona]:1},pending:member.persona,last_check_day:6};s.scene_subject=member.persona;
   await importState(page,s,language);await expect(page.locator('#main')).toHaveAttribute('data-screen','crew-care');
   const expected=[locales[language].trail.care_reason_0,locales[language].journey.care_body].map(t=>t.replaceAll('{name}',member.name)).join(' ');
   await expect.poll(()=>page.locator('.crew-incident > .scene-narrative').evaluate(el=>[...el.childNodes].filter(n=>n.nodeType===Node.TEXT_NODE).map(n=>n.textContent).join('').replace(/\s+/g,' ').trim())).toBe(expected);
   const dimensions=await layout(page);assert(dimensions.scrollWidth<=393+1);assert.deepEqual(dimensions.overflowingButtons,[]);
   if(language==='en'||language==='ar')await page.screenshot({path:out+`/care-${mode}-${language}-393.png`,fullPage:true});
   await page.locator('.crew-incident .camp-actions > button').first().click();
   await expect(page.locator('.aftermath-panel .outcome-copy')).toHaveText(locales[language].journey.care_helped.replaceAll('{name}',member.name));
   const after=await checkpoint(page);assert.equal(after.clock_minutes,s.clock_minutes+60);assert.equal(after.stats.supplies,8);assert.equal(after.crew_care.pending,null);
   await context.setOffline(true);await page.reload();await page.waitForFunction(()=>window.dystrailOffline?.state==='ready');sameState(await checkpoint(page),after);
   report.activation.push({...report.currentCase,exactNamedProseAndOutcome:true,noHorizontalOverflow:true,careMinutes:60,careSupplies:2,offlineSavedStateUnchanged:true});persist();await context.close();
  }
  assert.deepEqual(report.errors,[]);delete report.currentCase;persist();console.log(JSON.stringify({revision:manifest.revision,tariffCases:report.layouts.length,careCases:report.activation.length,errors:report.errors}));
 }catch(error){if(activePage&&!activePage.isClosed()){await activePage.screenshot({path:out+'/failure.png',fullPage:true});fs.writeFileSync(out+'/failure-body.txt',await activePage.locator('body').innerText())}throw error}finally{await browser.close()}
})().catch(error=>{report.failure=error.stack;persist();console.error(error);process.exit(1)});
