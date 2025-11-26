import { test, expect } from '@playwright/test';

test.describe('Visual Regression Tests', () => {
  test('landing-layout', async ({ page }) => {
    // Navigate to the paused simulation for stable screenshots
    await page.goto('/?paused=true');
    
    // Wait for the canvas to be attached
    await page.waitForSelector('#simulation', { state: 'attached' });
    
    // Give the canvas a moment to render
    await page.waitForTimeout(500);
    
    // Take a screenshot and compare with baseline
    await expect(page).toHaveScreenshot('landing-layout.png', {
      maxDiffPixels: 100, // Tolerate tiny font rendering differences
    });
  });

  test('canvas-exists', async ({ page }) => {
    // Navigate to the paused simulation
    await page.goto('/?paused=true');
    
    // Wait for the canvas to be attached
    const canvas = page.locator('#simulation');
    await canvas.waitFor({ state: 'attached' });
    
    // Check that the canvas has a width greater than 0
    const boundingBox = await canvas.boundingBox();
    expect(boundingBox).not.toBeNull();
    expect(boundingBox!.width).toBeGreaterThan(0);
    expect(boundingBox!.height).toBeGreaterThan(0);
  });
});

