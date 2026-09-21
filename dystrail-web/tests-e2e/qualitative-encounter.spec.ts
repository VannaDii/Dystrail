import {test,expect} from '@playwright/test';
import {baseline,importState,openMenu} from './helpers';

test('encounter previews stay qualitative in every language while costs still disable choices',async({page},info)=>{
 await page.setViewportSize({width:info.project.name==='mobile'?390:1440,height:1000});
 const state=await baseline(page);state.seed=42;state.stats.supplies=0;state.stats.hp=10;state.budget_cents=0;
 state.current_encounter={id:'preview_fixture',name:'Preview fixture',desc:'An offer before choosing.',weight:1,regions:[],modes:[],hard_stop:false,major_repair:false,chainable:false,choices:[
  {label:'Work',effects:{supplies:2,sanity:-1,add_receipt:'testimony',cash_cents:500}},
  {label:'Spend',effects:{supplies:-2,cash_cents:-500}},
  {label:'Recover',effects:{hp:2}}
 ]};
 await importState(page,state);
 const choices=page.locator('.encounter-choice button');await expect(choices).toHaveCount(3);
 await expect(choices.nth(0)).toBeEnabled();await expect(choices.nth(1)).toBeDisabled();
 await expect(choices.nth(2)).toContainText('already full');
 await openMenu(page);await page.locator('.language-picker > button').click();
 const languages=await page.getByRole('option').allTextContents();
 expect(languages.length).toBe(20);
 await page.keyboard.press('Escape');await page.locator('#game-menu-button').click();
 for(const language of languages){
  await openMenu(page);await page.locator('.language-picker > button').click();
  await page.getByRole('option',{name:language.trim(),exact:true}).click();await page.locator('#game-menu-button').click();
  for(const choice of await choices.all()){
   const hint=await choice.locator('small').innerText();const title=await choice.getAttribute('title');
   expect(hint,language).not.toMatch(/[0-9٠-٩۰-۹%+−]/);
   expect(hint,language).not.toContain('qualitative.');expect(hint,language).not.toContain('{stat}');
   expect(title).toBe(hint);
  }
  await expect(choices.nth(1)).toBeDisabled();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),language).toBe(true);
 }
});
