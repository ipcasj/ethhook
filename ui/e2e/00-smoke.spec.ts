import { test, expect } from '@playwright/test';
import { TestHelpers } from './fixtures/test-helpers';

/**
 * Critical Path Smoke Test
 * 
 * This is a fast smoke test that covers the most critical user journey:
 * Login → Create App → Create Endpoint → View Events
 * 
 * Run this before every deployment to catch critical issues quickly.
 */

test.describe('Critical Path Smoke Test', () => {
  test('complete user workflow: login → create app → create endpoint → monitor events', async ({ page }) => {
    const helpers = new TestHelpers(page);
    const timestamp = Date.now();
    
    // ========================================
    // 1. LOGIN WITH DEMO USER
    // ========================================
    console.log('Step 1: Login with demo user');
    await helpers.login('demo@ethhook.com', 'Demo1234!');
    await expect(page.locator('h1:has-text("Dashboard")')).toBeVisible();
    console.log('✓ Login successful');
    
    // ========================================
    // 2. CREATE APPLICATION
    // ========================================
    console.log('Step 2: Create Application');
    const appName = `Smoke Test App ${timestamp}`;
    await helpers.createApplication(appName, 'Critical path test application');
    // Verify we're on applications page with at least one app
    await expect(page.locator('h1:has-text("Applications")')).toBeVisible();
    await expect(page.locator('table tbody tr').first()).toBeVisible();
    console.log('✓ Application created');
    
    // ========================================
    // 3. CREATE ENDPOINT
    // ========================================
    console.log('Step 3: Create Endpoint');
    const endpointUrl = `https://webhook.example.com/smoke/${timestamp}`;
    await helpers.createEndpoint(appName, endpointUrl, [1]); // Ethereum Mainnet
    
    // Verify we're back on endpoints page with at least one endpoint
    await expect(page.locator('h1:has-text("Endpoints")')).toBeVisible();
    await expect(page.locator('table tbody tr').first()).toBeVisible(); // At least one row
    console.log('✓ Endpoint created');
    
    // ========================================
    // 4. VIEW EVENTS
    // ========================================
    console.log('Step 4: View Events');
    await helpers.navigateTo('Events');
    await expect(page.locator('h1:has-text("Events")')).toBeVisible();
    console.log('✓ Events page loaded');
    
    // ========================================
    // 5. VERIFY DATA PERSISTENCE
    // ========================================
    console.log('Step 5: Verify Data Persistence');
    
    // Go back to applications - verify table has rows
    await helpers.navigateTo('Applications');
    await expect(page.locator('table tbody tr').first()).toBeVisible();
    
    // Go to endpoints - verify table has rows
    await helpers.navigateTo('Endpoints');
    await expect(page.locator('table tbody tr').first()).toBeVisible();
    
    console.log('✓ Data persists across navigation');
    
    // ========================================
    // 6. LOGOUT
    // ========================================
    console.log('Step 6: Logout');
    await helpers.logout();
    await expect(page).toHaveURL('/login');
    console.log('✓ Logout successful');
    
    console.log('');
    console.log('🎉 SMOKE TEST PASSED - All critical features working!');
  });

  test('should handle network errors without crashing', async ({ page }) => {
    const helpers = new TestHelpers(page);
    
    console.log('Testing network error handling...');
    
    // Simulate backend down
    await page.route('**/api/**', route => route.abort());
    
    // Try to login
    await page.goto('/login');
    await page.fill('[data-testid="email-input"]', 'demo@ethhook.com');
    await page.fill('[data-testid="password-input"]', 'Demo1234!');
    await page.click('[data-testid="login-submit-button"]');
    
    // Should show error, NOT crash (this was the user's original bug!)
    const errorExists = await helpers.elementExists('text=/Network error|Failed to fetch|Unable to connect/i');
    expect(errorExists).toBeTruthy();
    
    console.log('✓ Network error handled gracefully - no crash!');
  });

  test('should load all pages without errors', async ({ page }) => {
    const helpers = new TestHelpers(page);
    
    // Login with demo user
    await helpers.login('demo@ethhook.com', 'Demo1234!');
    
    // Test all main pages
    const pages = [
      { name: 'Dashboard', heading: 'Dashboard' },
      { name: 'Applications', heading: 'Applications' },
      { name: 'Endpoints', heading: 'Endpoints' },
      { name: 'Events', heading: 'Events' },
    ];
    
    for (const pageInfo of pages) {
      console.log(`Testing ${pageInfo.name} page...`);
      await helpers.navigateTo(pageInfo.name);
      await expect(page.locator(`h1:has-text("${pageInfo.heading}")`)).toBeVisible();
      console.log(`✓ ${pageInfo.name} page loaded`);
    }
    
    console.log('');
    console.log('🎉 All pages load successfully!');
  });
});
