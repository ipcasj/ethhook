import { test, expect } from '@playwright/test';
import { TestHelpers } from './fixtures/test-helpers';

/**
 * Applications CRUD Tests
 * 
 * Tests the complete application lifecycle:
 * - Create application
 * - View application list
 * - View application details
 * - Update application
 * - Manage API keys
 * - Delete application
 */

test.describe('Applications Management', () => {
  let helpers: TestHelpers;

  test.beforeEach(async ({ page }) => {
    helpers = new TestHelpers(page);
    
    // Login before each test
    await helpers.login('demo@ethhook.com', 'Demo1234!');
  });

  test('should display applications list', async ({ page }) => {
    await helpers.navigateTo('Applications');
    
    // Wait for data to load
    await helpers.waitForLoading();
    
    // Check page elements
    await expect(page.locator('h1:has-text("Applications")')).toBeVisible();
    await expect(page.locator('button:has-text("Create Application")')).toBeVisible();
    
    // Should have table or empty state
    const hasTable = await helpers.elementExists('table');
    const hasEmptyState = await helpers.elementExists('text=No applications yet');
    
    expect(hasTable || hasEmptyState).toBeTruthy();
  });

  test('should create a new application', async ({ page }) => {
    const timestamp = Date.now();
    const appName = `Test App ${timestamp}`;
    const appDescription = 'Test application for E2E testing';
    
    await helpers.createApplication(appName, appDescription);
    
    // Should see the new application in the list
    await expect(page.locator(`text=${appName}`)).toBeVisible();
    await expect(page.locator(`text=${appDescription}`)).toBeVisible();
    
    // Should show API key after creation
    await expect(page.locator('text=API Key')).toBeVisible();
    await expect(page.locator('text=Make sure to copy your API key')).toBeVisible();
  });

  test('should show validation errors for invalid input', async ({ page }) => {
    await helpers.navigateTo('Applications');
    await page.click('button:has-text("Create Application")');
    
    // Try to submit empty form
    await page.click('button[type="submit"]');
    
    // Should show validation errors
    await expect(page.locator('text=/Name is required|Please enter a name/i')).toBeVisible();
  });

  test('should view application details', async ({ page }) => {
    await helpers.navigateTo('Applications');
    await helpers.waitForLoading();
    
    // Click on first application
    const firstApp = page.locator('table tbody tr').first();
    await firstApp.click();
    
    // Should show application details
    await expect(page.locator('text=Application Details')).toBeVisible();
    await expect(page.locator('text=API Key')).toBeVisible();
    await expect(page.locator('text=Endpoints')).toBeVisible();
  });

  test('should mask and reveal API key', async ({ page }) => {
    // Create a new application to get fresh API key
    const timestamp = Date.now();
    await helpers.createApplication(`Test App ${timestamp}`, 'Test description');
    
    // Check that API key is masked
    const maskedKey = page.locator('text=/••••••••/');
    await expect(maskedKey).toBeVisible();
    
    // Click reveal button
    await page.click('button[aria-label="Reveal API key"]');
    
    // Should show full key
    await expect(page.locator('text=/^eth_[a-zA-Z0-9]{32}$/')).toBeVisible();
  });

  test('should copy API key to clipboard', async ({ page, context }) => {
    // Create application
    const timestamp = Date.now();
    await helpers.createApplication(`Test App ${timestamp}`, 'Test description');
    
    // Grant clipboard permissions
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);
    
    // Click copy button
    await page.click('button[aria-label="Copy API key"]');
    
    // Should show success toast
    await helpers.waitForToast('API key copied');
  });

  test('should update application details', async ({ page }) => {
    // Create application first
    const timestamp = Date.now();
    const originalName = `Test App ${timestamp}`;
    await helpers.createApplication(originalName, 'Original description');
    
    // Click edit button
    await page.click('button:has-text("Edit")');
    
    // Update fields
    const updatedName = `Updated App ${timestamp}`;
    const updatedDescription = 'Updated description for testing';
    
    await page.fill('input#name', updatedName);
    await page.fill('input#description', updatedDescription);
    
    await page.click('button[type="submit"]');
    
    // Should show success message
    await helpers.waitForToast('Application updated');
    
    // Should see updated values
    await expect(page.locator(`text=${updatedName}`)).toBeVisible();
    await expect(page.locator(`text=${updatedDescription}`)).toBeVisible();
  });

  test('should delete application', async ({ page }) => {
    // Create application to delete
    const timestamp = Date.now();
    const appName = `App to Delete ${timestamp}`;
    await helpers.createApplication(appName, 'Will be deleted');
    
    // Navigate back to list
    await helpers.navigateTo('Applications');
    
    // Find and delete the application
    const row = page.locator(`tr:has-text("${appName}")`);
    await row.locator('button[aria-label="Delete"]').click();
    
    // Confirm deletion in modal
    await page.click('button:has-text("Delete")');
    
    // Should show success message
    await helpers.waitForToast('Application deleted');
    
    // Should not see the application anymore
    await expect(page.locator(`text=${appName}`)).not.toBeVisible();
  });

  test('should filter applications by search', async ({ page }) => {
    // Create two applications
    const timestamp = Date.now();
    await helpers.createApplication(`Searchable App ${timestamp}`, 'Description 1');
    await helpers.navigateTo('Applications');
    await helpers.createApplication(`Other App ${timestamp}`, 'Description 2');
    
    // Navigate to applications list
    await helpers.navigateTo('Applications');
    await helpers.waitForLoading();
    
    // Search for specific app
    await page.fill('input[placeholder="Search applications"]', 'Searchable');
    
    // Should only show matching application
    await expect(page.locator(`text=Searchable App ${timestamp}`)).toBeVisible();
    await expect(page.locator(`text=Other App ${timestamp}`)).not.toBeVisible();
  });

  test('should handle API errors gracefully', async ({ page }) => {
    await helpers.navigateTo('Applications');
    
    // Simulate API failure
    await page.route('**/api/applications', route => route.abort());
    
    // Reload to trigger API call
    await page.reload();
    
    // Should show error message, not crash
    await expect(page.locator('text=/Error loading|Failed to load|Unable to fetch/i')).toBeVisible();
  });

  test('should paginate applications list', async ({ page }) => {
    await helpers.navigateTo('Applications');
    await helpers.waitForLoading();
    
    // Check if pagination exists (only if there are many applications)
    const paginationExists = await helpers.elementExists('[aria-label="Pagination"]');
    
    if (paginationExists) {
      // Click next page
      await page.click('button[aria-label="Next page"]');
      
      // Should load next page
      await helpers.waitForLoading();
      
      // Page number should change
      await expect(page.locator('text=/Page 2|2 of/i')).toBeVisible();
    }
  });
});
