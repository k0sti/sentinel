import { test, expect } from '@playwright/test'

test('app renders with title', async ({ page }) => {
  await page.goto('/')
  await expect(page.locator('text=Sentinel')).toBeVisible()
})

test('start/stop tracking button visible', async ({ page }) => {
  await page.goto('/')
  const startButton = page.locator('button:has-text("Start")')
  await expect(startButton).toBeVisible()
})

test('settings tab shows form', async ({ page }) => {
  await page.goto('/')
  await page.click('text=Settings')
  await expect(page.locator('text=Tracking')).toBeVisible()
  await expect(page.locator('text=Interval')).toBeVisible()
})

test('identity tab shows add form', async ({ page }) => {
  await page.goto('/')
  await page.click('text=Identity')
  await expect(page.locator('text=Add Identity')).toBeVisible()
  await expect(page.locator('text=Add nsec')).toBeVisible()
})

test('settings saves config', async ({ page }) => {
  await page.goto('/')
  await page.click('text=Settings')

  const dTagInput = page.locator('input[type="text"]').first()
  await dTagInput.fill('my-phone')

  // Navigate away and back to verify persistence
  await page.click('text=Map')
  await page.click('text=Settings')

  // Config should be saved via localStorage
  const stored = await page.evaluate(() => localStorage.getItem('sentinel-tracking-config'))
  expect(stored).toContain('my-phone')
})
