import { test, expect } from '@playwright/test';

test.describe('Dystrail shell navigation + a11y smoke', () => {
  test('boot/menu flow, settings controls, and setup navigation', async ({ page }) => {
    await page.goto('./');

    // Boot screen and skip link exist
    await expect(page.locator('a[href="#main"]')).toHaveCount(1);
    await expect(page.locator('[data-testid="boot-screen"]')).toBeVisible();

    // Wait for ready prompt then advance
    await page.locator('[data-testid="boot-screen"] kbd').waitFor();
    await page.keyboard.press('Enter');

    await expect(page.locator('[data-testid="menu-screen"]')).toBeVisible();
    await expect(page).toHaveURL(/\/menu/);

    const menuItems = page.locator('[data-testid="menu-screen"] [role="menuitem"]');

    // Keyboard navigation: open About
    await menuItems.first().focus();
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('Enter');
    await expect(page.locator('[data-testid="about-screen"]')).toBeVisible();
    await expect(page).toHaveURL(/\/about/);
    await page.keyboard.press('Escape');
    await expect(page.locator('[data-testid="menu-screen"]')).toBeVisible();

    // Keyboard navigation: open Settings
    await menuItems.first().focus();
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('Enter');
    await expect(page.locator('[data-testid="settings-screen"]')).toBeVisible();
    await expect(page).toHaveURL(/\/settings/);

    // Language + dir toggle
    const html = page.locator('html');
    await page.locator('[data-testid="settings-language"]').selectOption('ar');
    await expect(html).toHaveAttribute('dir', 'rtl');
    await page.locator('[data-testid="settings-language"]').selectOption('en');
    await expect(html).toHaveAttribute('dir', 'ltr');

    // High-contrast toggle adds class on html
    await page.locator('[data-testid="settings-contrast"]').click();
    await expect(html).toHaveClass(/hc/);

    // Escape returns to menu
    await page.keyboard.press('Escape');
    await expect(page.locator('[data-testid="menu-screen"]')).toBeVisible();

    // Start journey -> persona -> mode -> outfitting -> travel
    await menuItems.first().focus();
    await page.keyboard.press('Enter');
    await expect(page).toHaveURL(/\/persona/);
    await page.locator('#persona-radios [data-key="1"]').click();
    await page.locator('#persona-continue').click();

    await expect(page.locator('[data-testid="mode-select"]')).toBeVisible();
    await page.locator('[data-testid="mode-classic"]').click();
    await page.locator('[data-testid="mode-continue"]').click();

    await expect(page.locator('[data-testid="outfitting-store"]')).toBeVisible();
    await page.locator('[data-testid="outfitting-store"]').focus();
    await page.keyboard.press('0');

    await expect(page.locator('.travel-shell')).toBeVisible();
    await expect(page).toHaveURL(/\/travel/);
  });
});
