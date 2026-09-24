import {test,expect} from '@playwright/test';
import {readFileSync,readdirSync} from 'node:fs';
import {waitForLaunch} from './helpers';
test('every supported locale shows its Deep End theme notice before selection',async({page},info)=>{
 test.setTimeout(180000);await page.goto('./');await waitForLaunch(page);
 for(const file of readdirSync('i18n').filter(f=>f.endsWith('.json'))){
  const lang=file.replace('.json',''),copy=JSON.parse(readFileSync(`i18n/${file}`,'utf8'));
  await page.evaluate(lang=>{localStorage.clear();localStorage.setItem('dystrail.locale',lang);},lang);await page.reload();await waitForLaunch(page);
  await expect(page.getByText(copy.ux.deep_desc,{exact:true})).toBeVisible();
  const radios=page.getByRole('radio');await expect(radios.first()).toBeChecked();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  if(['ar','ta'].includes(lang))await page.screenshot({path:info.outputPath(`mode-${lang}.png`),fullPage:true});
 }
});
