import {test, expect, Page} from '@playwright/test';
import {baseline, importState, openMenu, snap} from './helpers';
import {writeFileSync} from 'node:fs';

async function ending(page: Page) {
  const state = await baseline(page);
  state.abandoned = true;
  await importState(page, state);
  return state;
}

test('share dialog restores focus after mouse and keyboard close and releases each image', async ({page}) => {
  await ending(page);
  await page.evaluate(() => {
    const original = URL.revokeObjectURL;
    (window as any).releasedImages = [];
    URL.revokeObjectURL = url => { (window as any).releasedImages.push(url); original(url); };
  });
  for (const close of ['mouse', 'keyboard']) {
    await page.locator('#result-share-open').click();
    const dialog = page.getByRole('dialog');
    await expect(dialog.locator('.share-preview img')).toBeVisible();
    const image = await dialog.locator('.share-preview img').getAttribute('src');
    await expect(dialog.getByRole('button', {name:'Close', exact:true})).toBeFocused();
    await page.keyboard.press('Shift+Tab');
    await expect(dialog.locator('.share-actions > button:enabled, .share-actions > a[href]').last()).toBeFocused();
    await page.keyboard.press('Tab');
    await expect(dialog.getByRole('button', {name:'Close', exact:true})).toBeFocused();
    if (close === 'mouse') await dialog.getByRole('button', {name:'Close', exact:true}).click();
    else await page.keyboard.press('Escape');
    await expect(dialog).toHaveCount(0);
    await expect(page.locator('#result-share-open')).toBeFocused();
    await expect.poll(() => page.evaluate(url => (window as any).releasedImages.includes(url), image)).toBe(true);
  }
});

test('copy and native sharing use the edited post and handle cancellation, failure and handoff', async ({page}) => {
  await ending(page);
  await page.evaluate(() => {
    const state = (window as any).sharingProbe = {mode:'cancel', copied:'', calls:[]};
    Object.defineProperty(navigator, 'clipboard', {configurable:true, value:{writeText:(text:string) => {state.copied=text; return Promise.resolve();}}});
    Object.defineProperty(navigator, 'canShare', {configurable:true, value:(data:ShareData) => !!data.files?.length});
    Object.defineProperty(navigator, 'share', {configurable:true, value:(data:ShareData) => {
      state.calls.push({text:data.text, title:data.title, name:data.files?.[0].name, type:data.files?.[0].type, size:data.files?.[0].size, active:navigator.userActivation.isActive});
      return new Promise((resolve, reject) => {
        (state as any).finish = () => {
          (document.activeElement as HTMLElement)?.blur();
          if (state.mode === 'cancel') reject(new DOMException('Cancelled', 'AbortError'));
          else if (state.mode === 'fail') reject(new DOMException('Unavailable', 'NotAllowedError'));
          else resolve(undefined);
        };
      });
    }});
  });
  await page.locator('#result-share-open').click();
  const dialog = page.getByRole('dialog');
  const text = 'My road trip: 12345 & receipts, with a story to tell.';
  await dialog.getByLabel('Your post').fill(text);
  await dialog.getByRole('button', {name:'Copy post', exact:true}).click();
  expect(await page.evaluate(() => (window as any).sharingProbe.copied)).toBe(text);
  const share = dialog.getByRole('button', {name:'Share image & post', exact:true});
  for (const [mode, message] of [['cancel','Sharing cancelled.'], ['fail','Sharing is unavailable.'], ['success','Passed to your device']]) {
    await page.evaluate(mode => { (window as any).sharingProbe.mode=mode; }, mode);
    await share.click();
    await expect(share).toBeDisabled();
    await page.evaluate(() => (window as any).sharingProbe.finish());
    await expect(dialog.locator('.share-status')).toContainText(message);
    await expect(share).toBeEnabled();
    await expect(share).toBeFocused();
    await expect(dialog.getByLabel('Your post')).toHaveValue(text);
    await page.keyboard.press('Tab');
    await expect(dialog.getByRole('button', {name:'Close', exact:true})).toBeFocused();
    await page.keyboard.press('Shift+Tab');
    await expect(share).toBeFocused();
  }
  const calls = await page.evaluate(() => (window as any).sharingProbe.calls);
  expect(calls).toHaveLength(3);
  for (const call of calls) {
    expect(call).toMatchObject({text, title:'Dystopian Trail', name:'dystopian-trail.png', type:'image/png', active:true});
    expect(call.size).toBeGreaterThan(10000);
  }
  await page.keyboard.press('Escape');
  await expect(dialog).toHaveCount(0);
  await expect(page.locator('#result-share-open')).toBeFocused();
});

test('unsupported sharing and denied clipboard keep the image and editable post available', async ({page}) => {
  await ending(page);
  await page.evaluate(() => {
    Object.defineProperty(navigator, 'canShare', {configurable:true, value:() => false});
    Object.defineProperty(navigator, 'clipboard', {configurable:true, value:{writeText:() => Promise.reject(new DOMException('Denied', 'NotAllowedError'))}});
  });
  await page.locator('#result-share-open').click();
  const dialog = page.getByRole('dialog');
  await expect(dialog.locator('.share-preview img')).toBeVisible();
  await expect(dialog.getByRole('button', {name:'Share image & post', exact:true})).toHaveCount(0);
  await dialog.getByRole('button', {name:'Copy post', exact:true}).click();
  await expect(dialog.locator('.share-status')).toContainText('failed');
  await expect(dialog.getByRole('link', {name:'Save image', exact:true})).toBeVisible();
  await dialog.getByLabel('Your post').fill('Still editable');
  await expect(dialog.getByLabel('Your post')).toHaveValue('Still editable');
});

test('failed PNG encoding can retry without losing the edited post', async ({page}) => {
  await ending(page);
  await page.evaluate(() => {
    (window as any).originalPngEncoder=HTMLCanvasElement.prototype.toBlob;
    HTMLCanvasElement.prototype.toBlob = callback => callback(null);
  });
  await page.locator('#result-share-open').click();
  const dialog = page.getByRole('dialog');
  await expect(dialog.getByRole('alert')).toContainText('share image could not be created');
  await dialog.getByLabel('Your post').fill('Keep my edited story during retry.');
  await page.evaluate(() => { HTMLCanvasElement.prototype.toBlob=(window as any).originalPngEncoder; });
  await dialog.getByRole('button', {name:'Try again', exact:true}).click();
  await expect(dialog.locator('.share-preview img')).toBeVisible();
  await expect(dialog.getByLabel('Your post')).toHaveValue('Keep my edited story during retry.');
});

test('closing during image preparation releases the completed image without reopening the dialog', async ({page}) => {
  await ending(page);
  await page.evaluate(() => {
    const original=HTMLCanvasElement.prototype.toBlob;
    const create=URL.createObjectURL, revoke=URL.revokeObjectURL;
    (window as any).pngLifecycle={created:[], released:[]};
    URL.createObjectURL = blob => {const url=create(blob); (window as any).pngLifecycle.created.push(url); return url;};
    URL.revokeObjectURL = url => {(window as any).pngLifecycle.released.push(url); revoke(url);};
    HTMLCanvasElement.prototype.toBlob = function(callback, type, quality) {
      (window as any).finishPendingImage = () => original.call(this, callback, type, quality);
    };
  });
  await page.locator('#result-share-open').click();
  await expect(page.locator('.share-placeholder')).toContainText('Preparing');
  await page.getByRole('dialog').getByRole('button', {name:'Close', exact:true}).click();
  await page.evaluate(() => (window as any).finishPendingImage());
  await expect.poll(() => page.evaluate(() => (window as any).pngLifecycle.created.length)).toBe(1);
  expect(await page.evaluate(() => (window as any).pngLifecycle.released)).toEqual(await page.evaluate(() => (window as any).pngLifecycle.created));
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await expect(page.locator('#result-share-open')).toBeFocused();
});

test('long player names and Arabic share content stay readable on desktop and phone', async ({page}, info) => {
  const state=await ending(page);
  const player=state.party.members.find((member:any) => member.persona===state.persona_id);
  player.name='Alexandria River Song With An Exceptionally Long Family Name';
  await importState(page,state);
  await openMenu(page);
  await page.locator('.language-picker>button').click();
  await page.getByRole('option', {name:'العربية',exact:true}).click();
  await page.locator('#game-menu-button').click();
  await page.locator('#result-share-open').click();
  const image=page.locator('.share-preview img');
  await expect(image).toBeVisible();
  await expect(page.locator('html')).toHaveAttribute('dir','rtl');
  await expect(page.locator('#share-post')).toHaveValue(new RegExp(player.name));
  expect(await image.evaluate((img:HTMLImageElement) => [img.naturalWidth,img.naturalHeight])).toEqual([1200,1200]);
  const png = await image.evaluate(async (img:HTMLImageElement) => Array.from(new Uint8Array(await (await fetch(img.src)).arrayBuffer())));
  writeFileSync(info.outputPath('arabic-share.png'), Buffer.from(png));
  await snap(page,'share-arabic-long-name');
});
