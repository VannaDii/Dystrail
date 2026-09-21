import {test,expect} from '@playwright/test';
import {readFileSync} from 'node:fs';
import {baseline,importState,openMenu,waitForLaunch} from './helpers';
const event=JSON.parse(readFileSync('static/assets/data/game.json','utf8')).find((e:any)=>e.id==='classic_bridge_crews');

test('C01 variants keep copy, art, committed choices and saved identity aligned',async({page,context},info)=>{
 await page.setViewportSize({width:info.project.name==='mobile'?390:1440,height:1000});
 await page.emulateMedia({reducedMotion:'reduce'});
 test.setTimeout(120000);
 const base=await baseline(page);base.seed=42;base.region='RustBelt';base.stats.supplies=10;base.stats.hp=9;base.stats.credibility=5;
 base.current_encounter=event;base.last_encounter_driving_minutes=300;base.driving_minutes_total=300;
 const titles=['Smaller government','Permission to spin','Dignity, by the pound'];
 for(let row=0;row<3;row++){
  const unit=`ENC-C01-${'ABC'[row]}`;
  for(let choice=0;choice<2;choice++){
   const state=structuredClone(base);state.visual_content={edition:1,selections:{'ENC-C01/road/300':unit}};
   await importState(page,state);
   await expect(page.locator('#screen-title')).toContainText(titles[row]);
   const scene=page.locator('.scene-atlas[data-atlas="road-c01-20260914"]');
   await expect(scene).toHaveAttribute('data-cell',String(row*2));
   await page.reload();await waitForLaunch(page);
   await expect(page.locator('#screen-title')).toContainText(titles[row]);
   await expect(scene).toHaveAttribute('data-cell',String(row*2));
   if(row>0 && choice===0){
    await page.getByRole('button',{name:titles[row],exact:true}).click();
    await expect(page.locator('.viewport-help:visible')).toContainText(row===1?'PART NOT RECOGNIZED':'CALIBRI');
    await page.keyboard.press('Escape');
   }
   if(choice===0){
    await page.evaluate(()=>{(document.activeElement as HTMLElement)?.blur();window.scrollTo(0,0);});
    await page.screenshot({path:info.outputPath(`${unit}-offer.png`),fullPage:true});
   }
   await page.locator('.encounter-choice button').nth(choice).click();
   await expect(scene).toHaveAttribute('data-cell',String(row*2+(choice===0?1:0)));
   await expect(page.locator('#screen-title')).toContainText(titles[row]);
   await openMenu(page);await page.getByRole('button',{name:'Save',exact:true}).click();
   const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('dystrail.save.default')!));
   expect(saved.visual_content.selections['ENC-C01/road/300']).toBe(unit);
   expect(saved.current_encounter).toBeNull();
   expect(saved.stats.supplies).toBe(choice===0?9:10);expect(saved.stats.hp).toBe(choice===0?8:9);
   expect(saved.stats.credibility).toBe(choice===0?7:6);
   expect(saved.visual_content.outcomes['ENC-C01/road/300']).toBe(choice);
   expect(saved.journal.at(-1).title).toBe(titles[row]);
  }
 }
 const state=structuredClone(base);state.visual_content={edition:1,selections:{'ENC-C01/road/300':'ENC-C01-B'}};
 await importState(page,state);
 await openMenu(page);await page.locator('.language-picker > button').click();await page.getByRole('option',{name:'العربية',exact:true}).click();await page.locator('#game-menu-button').click();
 await expect(page.locator('#screen-title')).not.toContainText('Permission to spin');
 await context.setOffline(true);await page.reload();await waitForLaunch(page);
 await expect(page.locator('.scene-atlas[data-atlas="road-c01-20260914"]')).toHaveAttribute('data-cell','2');
 expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
});
