import {test,expect} from '@playwright/test';
import {baseline,importState,savedState,snap} from './helpers';
test('player and crew names recover during onboarding and absent members leave their seats',async({page})=>{
 await page.goto('./');await page.getByRole('button',{name:'Choose your character',exact:true}).click();await page.getByRole('radio',{name:'Journalist',exact:true}).click();await page.getByRole('button',{name:'Continue',exact:true}).click();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','crew');await expect(page.getByRole('button',{name:'Continue',exact:true})).toBeDisabled();
 const names=await page.locator('.crew-name-card input').evaluateAll(inputs=>inputs.map(input=>(input as HTMLInputElement).value));expect(names.filter(Boolean)).toHaveLength(5);expect(new Set(names).size).toBe(6);
 await page.getByLabel('Your name',{exact:true}).fill('Vanna Full Name');await page.getByLabel('Crew name',{exact:true}).fill('The Good Trouble');await page.getByLabel('Organizer',{exact:true}).fill('Alex Morgan');
 await page.reload();await expect(page.getByLabel('Your name',{exact:true})).toHaveValue('Vanna Full Name');await expect(page.getByLabel('Organizer',{exact:true})).toHaveValue('Alex Morgan');await snap(page,'crew-names');
 await page.getByRole('button',{name:'Continue',exact:true}).click();await page.getByRole('button',{name:'Review & depart',exact:true}).filter({visible:true}).first().click();await page.getByRole('button',{name:'Start the journey',exact:true}).click();
 const state=await savedState(page);expect(state.party.leader).toBe('Vanna Full Name');expect(state.party.name).toBe('The Good Trouble');await expect(page.locator('.van-occupant')).toHaveCount(6);
 state.party.members.find((m:{persona:string})=>m.persona==='satirist').status='Dead';state.party.members.find((m:{persona:string})=>m.persona==='organizer').status='Departed';await importState(page,state);
 await expect(page.locator('.van-occupant')).toHaveCount(4);await expect(page.locator('.van-occupant[data-member=satirist]')).toHaveCount(0);await expect(page.locator('.van-occupant[data-member=organizer]')).toHaveCount(0);await expect(page.locator('.van-occupant[data-seat="4"]')).toHaveCount(1);
 await page.getByRole('button',{name:'In the van',exact:true}).click();await expect(page.locator('.crew-roster')).toContainText('Died');await expect(page.locator('.crew-roster')).toContainText('Left the crew');await page.reload();await expect(page.locator('.van-occupant')).toHaveCount(4);await snap(page,'crew-absences');
 state.ending={type:'collapse',cause:'panic'};await importState(page,state);
 await expect(page.locator('#main')).toHaveAttribute('data-screen','result');
 await expect(page.locator('.ending-crew [data-fate="crew.departed"]')).toContainText('Alex Morgan');
 await expect(page.locator('.scene-speaker[data-subject=organizer]')).toHaveCount(0);
});
