import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('landing page works without console errors and has no serious accessibility issues', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
  await page.goto('/');
  await expect(page).toHaveTitle(/Scheduled Run Receipts/);
  await expect(page.locator('main')).toBeVisible();
  await expect(page.locator('h1')).toHaveCount(1);
  await page.getByRole('radio', { name: /Wed Missing/ }).click();
  await expect(page.locator('#receipt-detail')).toContainText('No signed start arrived');
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter((item) => ['serious', 'critical'].includes(item.impact ?? ''))).toEqual([]);
  expect(errors).toEqual([]);
});

test('evidence slots use one roving tab stop and every visible control fits the viewport', async ({ page }) => {
  await page.goto('/');
  const slots = page.getByRole('radio');
  await expect(slots).toHaveCount(7);
  await expect(page.locator('[role="radio"][tabindex="0"]')).toHaveCount(1);
  await slots.last().focus();
  await page.keyboard.press('ArrowRight');
  await expect(slots.first()).toBeFocused();
  await expect(page.locator('[role="radio"][aria-checked="true"]')).toHaveCount(1);
  await expect(page.locator('[role="radio"][tabindex="0"]')).toHaveCount(1);

  const tooSmall = await page.locator('a, button, label.file-label').evaluateAll((elements) => elements
    .filter((element) => {
      const style = getComputedStyle(element);
      const rect = element.getBoundingClientRect();
      return style.visibility !== 'hidden' && style.display !== 'none' && rect.width > 0 && rect.height > 0;
    })
    .map((element) => ({ label: element.textContent?.trim(), ...element.getBoundingClientRect().toJSON() }))
    .filter((rect) => rect.width < 44 || rect.height < 44));
  expect(tooSmall).toEqual([]);
  const widths = await page.evaluate(() => ({ client: document.documentElement.clientWidth, scroll: document.documentElement.scrollWidth }));
  expect(widths.scroll).toBe(widths.client);
});

test('viewer handles a local empty report and legal routes resolve', async ({ page }) => {
  await page.goto('/');
  await page.locator('#report-file').setInputFiles({ name: 'empty.json', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify({ generated_at: new Date().toISOString(), since: '', healthy: true, counts: {}, slots: [] })) });
  await expect(page.locator('.viewer-empty')).toContainText('No expected slots');
  await page.goto('/privacy/');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Privacy');
  await page.goto('/terms/');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Terms');
});

test('@claim:offline-reload works offline after the first visit', async ({ page }) => {
  await page.goto('/');
  await page.evaluate(async () => { await navigator.serviceWorker.ready; });
  await page.context().setOffline(true);
  await page.reload();
  await expect(page).toHaveTitle(/Scheduled Run Receipts/);
  await expect(page.locator('main')).toBeVisible();
  await page.context().setOffline(false);
});

test('@claim:local-viewer-private loads a report without third-party requests', async ({ page }) => {
  const requestedOrigins = new Set<string>();
  page.on('request', (request) => requestedOrigins.add(new URL(request.url()).origin));
  await page.goto('/');
  await page.locator('#report-file').setInputFiles({
    name: 'empty.json',
    mimeType: 'application/json',
    buffer: Buffer.from(JSON.stringify({ generated_at: new Date().toISOString(), since: '', healthy: true, counts: {}, slots: [] })),
  });
  await expect(page.locator('#file-status')).toContainText('Loaded empty.json locally. Nothing was uploaded.');
  expect([...requestedOrigins]).toEqual([new URL(page.url()).origin]);
});
