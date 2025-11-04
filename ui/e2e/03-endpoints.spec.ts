import { test, expect } from '@playwright/test';
import { TestHelpers } from './fixtures/test-helpers';

/**
 * Endpoints CRUD Tests
 * 
 * Tests webhook endpoint management:
 * - Create endpoints with multiple chains
 * - View endpoints list
 * - Update endpoints
 * - Test endpoint connectivity
 * - Delete endpoints
 */

test.describe('Endpoints Management', () => {
  let helpers: TestHelpers;
  let testAppName: string;

  test.beforeEach(async ({ page }) => {
    helpers = new TestHelpers(page);
    
    // Login and create test application
    await helpers.login('demo@ethhook.com', 'Demo1234!');
    
    const timestamp = Date.now();
    testAppName = `Test App ${timestamp}`;
    await helpers.createApplication(testAppName, 'For endpoint testing');
  });

  test('should display endpoints page', async ({ page }) => {
    await helpers.navigateTo('Endpoints');
    
    await expect(page.locator('h1:has-text("Endpoints")')).toBeVisible();
    await expect(page.locator('button:has-text("Add Endpoint")')).toBeVisible();
  });

  test('should create endpoint with single chain', async ({ page }) => {
    const timestamp = Date.now();
    const endpointUrl = `https://webhook.example.com/${timestamp}`;
    
    await helpers.createEndpoint(testAppName, endpointUrl, [1]); // Ethereum Mainnet
    
    // Should see the new endpoint
    await expect(page.locator(`text=${endpointUrl}`)).toBeVisible();
    await expect(page.locator('text=Ethereum Mainnet')).toBeVisible();
  });

  test('should create endpoint with multiple chains', async ({ page }) => {
    const timestamp = Date.now();
    const endpointUrl = `https://webhook.example.com/multi/${timestamp}`;
    
    await helpers.navigateTo('Endpoints');
    await page.click('button:has-text("Add Endpoint")');
    
    // Select application from dropdown
    await page.selectOption('select#application', { label: testAppName });
    
    // Enter URL
    await page.fill('input#webhookUrl', endpointUrl);
    
    // Select multiple chains (buttons, not labels/checkboxes)
    await page.click('button:has-text("Ethereum Mainnet")');
    await page.click('button:has-text("Sepolia")');
    await page.click('button:has-text("Polygon")');
    
    await page.click('button[type="submit"]');
    
    // Should show success and display all chains
    await helpers.waitForToast('Endpoint created successfully');
    await expect(page.locator('text=Ethereum Mainnet')).toBeVisible();
    await expect(page.locator('text=Sepolia')).toBeVisible();
    await expect(page.locator('text=Polygon')).toBeVisible();
  });

  test('should validate endpoint URL format', async ({ page }) => {
    await helpers.navigateTo('Endpoints');
    await page.click('button:has-text("Add Endpoint")');
    
    // Select application
    await page.selectOption('select#application', { label: testAppName });
    
    // Enter invalid URL
    await page.fill('input#webhookUrl', 'not-a-valid-url');
    
    // Select a chain
    await page.click('button:has-text("Ethereum Mainnet")');
    
    await page.click('button[type="submit"]');
    
    // Should show validation error
    await expect(page.locator('text=/Invalid URL|Please enter a valid URL/i')).toBeVisible();
  });

  test('should require at least one chain', async ({ page }) => {
    await helpers.navigateTo('Endpoints');
    await page.click('button:has-text("Add Endpoint")');
    
    // Select application
    await page.selectOption('select#application', { label: testAppName });
    
    // Enter URL but don't select any chains
    await page.fill('input#webhookUrl', 'https://webhook.example.com/test');
    
    await page.click('button[type="submit"]');
    
    // Should show validation error
    await expect(page.locator('text=/Select at least one chain|Chain is required/i')).toBeVisible();
  });

  test('should update endpoint', async ({ page }) => {
    // Create endpoint first
    const timestamp = Date.now();
    const originalUrl = `https://webhook.example.com/original/${timestamp}`;
    await helpers.createEndpoint(testAppName, originalUrl, [1]);
    
    // Click edit button
    await page.click('button:has-text("Edit")');
    
    // Update URL
    const updatedUrl = `https://webhook.example.com/updated/${timestamp}`;
    await page.fill('input#webhookUrl', updatedUrl);
    
    // Add another chain
    await page.click('label:has-text("Sepolia")');
    
    await page.click('button[type="submit"]');
    
    // Should show success
    await helpers.waitForToast('Endpoint updated');
    
    // Should see updated values
    await expect(page.locator(`text=${updatedUrl}`)).toBeVisible();
    await expect(page.locator('text=Sepolia')).toBeVisible();
  });

  test('should test endpoint connectivity', async ({ page }) => {
    const timestamp = Date.now();
    const endpointUrl = `https://webhook.example.com/test/${timestamp}`;
    
    await helpers.createEndpoint(testAppName, endpointUrl, [1]);
    
    // Click test button
    await page.click('button:has-text("Test Endpoint")');
    
    // Should send test webhook
    await helpers.waitForLoading();
    
    // Should show result (success or failure)
    const testResult = page.locator('text=/Test successful|Test failed|Connection timeout/i');
    await expect(testResult).toBeVisible({ timeout: 10000 });
  });

  test('should delete endpoint', async ({ page }) => {
    const timestamp = Date.now();
    const endpointUrl = `https://webhook.example.com/delete/${timestamp}`;
    
    await helpers.createEndpoint(testAppName, endpointUrl, [1]);
    
    // Delete the endpoint
    const row = page.locator(`tr:has-text("${endpointUrl}")`);
    await row.locator('button[aria-label="Delete"]').click();
    
    // Confirm deletion
    await page.click('button:has-text("Delete")');
    
    // Should show success
    await helpers.waitForToast('Endpoint deleted');
    
    // Should not see endpoint anymore
    await expect(page.locator(`text=${endpointUrl}`)).not.toBeVisible();
  });

  test('should filter endpoints by chain', async ({ page }) => {
    // Create endpoints on different chains
    const timestamp = Date.now();
    await helpers.createEndpoint(testAppName, `https://webhook.example.com/eth/${timestamp}`, [1]);
    await helpers.navigateTo('Endpoints');
    await helpers.createEndpoint(testAppName, `https://webhook.example.com/poly/${timestamp}`, [137]);
    
    // Filter by Ethereum
    await page.click('button:has-text("Filter by Chain")');
    await page.click('text=Ethereum Mainnet');
    
    // Should only show Ethereum endpoint
    await expect(page.locator(`text=https://webhook.example.com/eth/${timestamp}`)).toBeVisible();
    await expect(page.locator(`text=https://webhook.example.com/poly/${timestamp}`)).not.toBeVisible();
  });

  test('should show endpoint statistics', async ({ page }) => {
    const timestamp = Date.now();
    await helpers.createEndpoint(testAppName, `https://webhook.example.com/stats/${timestamp}`, [1]);
    
    // Click on endpoint to view details
    await page.click(`text=https://webhook.example.com/stats/${timestamp}`);
    
    // Should show statistics
    await expect(page.locator('text=Total Deliveries')).toBeVisible();
    await expect(page.locator('text=Success Rate')).toBeVisible();
    await expect(page.locator('text=Average Response Time')).toBeVisible();
  });

  test('should handle endpoint creation failures', async ({ page }) => {
    await helpers.navigateTo('Endpoints');
    
    // Simulate API failure
    await page.route('**/api/endpoints', route => route.abort());
    
    await page.click('button:has-text("Add Endpoint")');
    
    // Fill form
    await page.selectOption('select#application', { label: testAppName });
    await page.fill('input#webhookUrl', 'https://webhook.example.com/test');
    await page.click('button:has-text("Ethereum Mainnet")');
    
    await page.click('button[type="submit"]');
    
    // Should show error, not crash
    await expect(page.locator('text=/Error creating|Failed to create|Network error/i')).toBeVisible();
  });
});
