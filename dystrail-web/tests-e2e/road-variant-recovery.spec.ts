import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,openMenu,waitForLaunch} from './helpers';
const bank=JSON.parse(readFileSync('static/assets/data/game.json','utf8'));
for(const family of ['C01','C02','C03','C04','C05','C06']) {
const event=bank.find((e:any)=>e.id===(family==='C01'?'classic_bridge_crews':family==='C02'?'classic_civic_potluck':family==='C03'?'classic_crossing_block_party':family==='C04'?'classic_freeway_mural':family==='C05'?'classic_mail_drop':'classic_media_training'));

test(`${family} variants keep copy, art, committed choices and saved identity aligned`,async({page,context},info)=>{
 await page.setViewportSize({width:info.project.name==='mobile'?390:1440,height:1000});
 await page.emulateMedia({reducedMotion:'reduce'});
 test.setTimeout(120000);
 const base=await baseline(page);base.seed=42;base.region='RustBelt';base.stats.supplies=10;base.stats.hp=9;base.stats.credibility=5;
 base.current_encounter=event;base.last_encounter_driving_minutes=300;base.driving_minutes_total=300;
 const copy=JSON.parse(readFileSync('i18n/en.json','utf8')).encounter_copy;
 const titles='ABC'.split('').map(suffix=>copy[`ENC-${family}-${suffix}`].name);
 for(let row=0;row<3;row++){
  const unit=`ENC-${family}-${'ABC'[row]}`;
  for(let choice=0;choice<event.choices.length;choice++){
   const state=structuredClone(base);state.visual_content={edition:1,selections:{[`ENC-${family}/road/300`]:unit}};
   await importState(page,state);
   await expect(page.locator('#screen-title')).toContainText(titles[row]);
   const atlas=family==='C01'?'road-c01-20260914':`road-${family.toLowerCase()}-${'abc'[row]}-20260914`;
   const held=(family==='C02'||family==='C04') && row===0;
   const scene=held?page.locator('.scene-atlas').first():family==='C02'?page.locator('.scene-atlas[data-atlas^="road-c02-"]'):page.locator(`.scene-atlas[data-atlas="${atlas}"]`);
   if(held) await expect(page.locator(`image[href*="road-${family.toLowerCase()}"]`)).toHaveCount(0);
   if(!held) await expect(scene).toHaveAttribute('data-cell',String(family==='C01'||family==='C02'?row*2:0));
   await page.reload();await waitForLaunch(page);
   await expect(page.locator('#screen-title')).toContainText(titles[row]);
   if(!held) await expect(scene).toHaveAttribute('data-cell',String(family==='C01'||family==='C02'?row*2:0));
   if(family==='C01' && row>0 && choice===0){
    await page.getByRole('button',{name:titles[row],exact:true}).click();
    await expect(page.locator('.viewport-help:visible')).toContainText(row===1?'PART NOT RECOGNIZED':'CALIBRI');
    await page.keyboard.press('Escape');
   }
   if(choice===0){
    await expect(page.locator('.scene-caption')).toHaveCSS('padding-right',info.project.name==='mobile'?'14px':'20px');
    await page.evaluate(()=>{(document.activeElement as HTMLElement)?.blur();window.scrollTo(0,0);});
    await page.screenshot({path:info.outputPath(`${unit}-offer.png`),fullPage:true});
   }
   await page.locator('.encounter-choice button').nth(choice).click();
   if(!held) await expect(scene).toHaveAttribute('data-cell',String(family==='C02'?row*2+(choice===0||choice===2?1:0):family==='C01'?row*2+(choice===0?1:0):choice+1));
   else await expect(scene).toBeVisible();
   if(family==='C02'&&!held) await expect(scene).toHaveAttribute('data-atlas',`road-c02-${choice===0?'offer-meal':'record-donate'}-20260914`);
   await expect(page.locator('#screen-title')).toContainText(titles[row]);
   if(family==='C02'||family==='C04'||family==='C05'||family==='C06') await page.screenshot({path:info.outputPath(`${unit}-outcome-${choice}.png`),fullPage:true});
   await openMenu(page);await page.getByRole('button',{name:'Save',exact:true}).click();
   const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.save.default')!));
   expect(saved.visual_content.selections[`ENC-${family}/road/300`]).toBe(unit);
   expect(saved.current_encounter).toBeNull();
   for(const stat of ['supplies','hp','credibility','morale','sanity']) {
    expect(saved.stats[stat]).toBe(Math.min(stat==='supplies'?20:10,base.stats[stat]+(event.choices[choice].effects[stat]??0)));
   }
   expect(saved.visual_content.outcomes[`ENC-${family}/road/300`]).toBe(choice);
   expect(saved.journal.at(-1).title).toBe(titles[row]);
  }
 }
 const state=structuredClone(base);state.visual_content={edition:1,selections:{[`ENC-${family}/road/300`]:`ENC-${family}-B`}};
 await importState(page,state);
 await openMenu(page);await page.locator('.language-picker > button').click();await page.getByRole('option',{name:'العربية',exact:true}).click();await page.locator('#game-menu-button').click();
 await expect(page.locator('#screen-title')).not.toContainText(titles[1]);
 if(info.project.name==='mobile') await page.setViewportSize({width:320,height:1000});
 await expect(page.locator('.scene-caption')).toHaveCSS('text-align','left');
 await context.setOffline(true);await page.reload();await waitForLaunch(page);
 await page.screenshot({path:info.outputPath('arabic-offline.png'),fullPage:true});
 await expect(page.locator(`.scene-atlas[data-atlas="${family==='C01'?'road-c01-20260914':family==='C02'?'road-c02-offer-meal-20260914':`road-${family.toLowerCase()}-b-20260914`}"]`)).toHaveAttribute('data-cell',family==='C01'||family==='C02'?'2':'0');
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});

}
