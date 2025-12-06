import { test, expect } from '@playwright/test';

test.describe('Dystrail a11y + i18n smoke', () => {
  test('loads home, supports skip link, lang/dir toggle, and high contrast', async ({ page }) => {
    await page.goto('/');

    // Main landmarks and skip link exist
    await expect(page.locator('a[href="#main"]')).toHaveCount(1);
    const main = page.locator('main#main');
    await expect(main).toBeVisible();

    // Default language and direction
    const html = page.locator('html');
    await expect(html).toHaveAttribute('lang', /en/i);
    await expect(html).toHaveAttribute('dir', /ltr/i);

    // Switch to Arabic (RTL) and back
    await page.locator('#lang-select').selectOption('ar');
    await expect(html).toHaveAttribute('dir', 'rtl');
    await page.locator('#lang-select').selectOption('en');
    await expect(html).toHaveAttribute('dir', 'ltr');

    // High-contrast toggle adds class on html
    const hcToggle = page.getByRole('button', { name: /HC/i });
    await hcToggle.click();
    await expect(html).toHaveClass(/hc/);
  });
});
