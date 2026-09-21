import {test,expect} from '@playwright/test';
import { baseline,importState,savedState,snap, waitForLaunch } from './helpers';
test('player and crew names recover during onboarding and absent members leave their seats',async({page})=>{
 await page.goto('./');await page.getByRole('button',{name:'Choose your character',exact:true}).click();await page.getByRole('radio',{name:'Journalist',exact:true}).click();await page.getByRole('button',{name:'Continue',exact:true}).click();
 await expect(page.locator('#main')).toHaveAttribute('data-screen','crew');await expect(page.getByRole('button',{name:'Continue',exact:true})).toBeDisabled();
 const names=await page.locator('.crew-name-card input').evaluateAll(inputs=>inputs.map(input=>(input as HTMLInputElement).value));expect(names.filter(Boolean)).toHaveLength(5);expect(new Set(names).size).toBe(6);
 await page.getByLabel('Your name',{exact:true}).fill('Vanna Full Name');await page.getByLabel('Crew name',{exact:true}).fill('The Good Trouble');await page.getByLabel('Organizer',{exact:true}).fill('Alex Morgan');
 await page.reload();await waitForLaunch(page);await expect(page.getByLabel('Your name',{exact:true})).toHaveValue('Vanna Full Name');await expect(page.getByLabel('Organizer',{exact:true})).toHaveValue('Alex Morgan');await snap(page,'crew-names');
 await page.getByRole('button',{name:'Continue',exact:true}).click();await page.getByRole('button',{name:'Review & depart',exact:true}).filter({visible:true}).first().click();await page.getByRole('button',{name:'Start the journey',exact:true}).click();
 const state=await savedState(page);expect(state.party.leader).toBe('Vanna Full Name');expect(state.party.name).toBe('The Good Trouble');await expect(page.locator('.standing-member')).toHaveCount(6);
 state.party.members.find((m:{persona:string})=>m.persona==='satirist').status='Dead';state.party.members.find((m:{persona:string})=>m.persona==='organizer').status='Departed';await importState(page,state);
 await expect(page.locator('.standing-member')).toHaveCount(4);await expect(page.locator('.standing-member[data-member=satirist]')).toHaveCount(0);await expect(page.locator('.standing-member[data-member=organizer]')).toHaveCount(0);await expect(page.locator('.standing-member[data-member=journalist]')).toHaveCount(1);
 await page.getByRole('tab',{name:'The Van',exact:true}).click();await expect(page.locator('.van-crew')).toContainText('Died');await expect(page.locator('.van-crew')).toContainText('Left the crew');await page.reload();await waitForLaunch(page);await expect(page.locator('.standing-member')).toHaveCount(4);await snap(page,'crew-absences');
 state.ending={type:'collapse',cause:'panic'};await importState(page,state);
 await expect(page.locator('#main')).toHaveAttribute('data-screen','result');
 await expect(page.locator('.ending-crew [data-fate="crew.departed"]')).toContainText('Alex Morgan');
 await expect(page.locator('.scene-speaker[data-subject=organizer]')).toHaveCount(0);
});

test('standing sprites have clear margins at full and fractional display sizes',async({page})=>{
 await baseline(page);
 // Use the actual scene SVGs and filter over a flat backing. Checking only that
 // the six images exist missed a translucent magenta matte on accelerated GPUs.
 for(const width of [128,64,41.5]){
  await page.evaluate(width=>{
   document.querySelector('#crew-transparency-probe')?.remove();
   const probe=document.createElement('div');probe.id='crew-transparency-probe';
   probe.style.cssText=`position:fixed;inset:0 auto auto 0;z-index:99999;display:grid;grid-template-columns:repeat(3,${width}px);background:rgb(85,102,119)`;
   for(const source of document.querySelectorAll('.standing-member')){
    const sprite=source.cloneNode(true) as SVGElement;sprite.removeAttribute('class');
    sprite.style.cssText=`display:block;width:${width}px;height:${width*1.6}px`;
    probe.append(sprite);
   }
   document.body.append(probe);
  },width);
  const pixels=await page.locator('#crew-transparency-probe').screenshot({animations:'disabled'});
  const samples=await page.evaluate(async data=>{
   const image=new Image();image.src=`data:image/png;base64,${data}`;await image.decode();
   const canvas=document.createElement('canvas');canvas.width=image.width;canvas.height=image.height;
   const context=canvas.getContext('2d')!;context.drawImage(image,0,0);
   return Array.from({length:6},(_,index)=>[.02,.98].flatMap(x=>[.1,.9].map(y=>{
    const px=Math.floor((index%3+x)*image.width/3);
    const py=Math.floor((Math.floor(index/3)+y)*image.height/2);
    return [...context.getImageData(px,py,1,1).data];
   })));
  },pixels.toString('base64'));
  for(const [member,points] of samples.entries())for(const sample of points){
   expect(sample.map((value,index)=>Math.abs(value-[85,102,119,255][index])),`member ${member}, width ${width}: ${sample}`).toEqual([0,0,0,0]);
  }
 }
 await page.locator('#crew-transparency-probe').evaluate(element=>element.remove());
 await snap(page,'crew-transparency');
});
