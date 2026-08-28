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

test('viewer handles a local empty report and legal routes resolve', async ({ page }) => {
  await page.goto('/');
  await page.locator('#report-file').setInputFiles({ name: 'empty.json', mimeType: 'application/json', buffer: Buffer.from(JSON.stringify({ generated_at: new Date().toISOString(), since: '', healthy: true, counts: {}, slots: [] })) });
  await expect(page.locator('.viewer-empty')).toContainText('No expected slots');
  await page.goto('/privacy/');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Privacy');
  await page.goto('/terms/');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Terms');
});
