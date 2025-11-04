import { test, expect } from '@playwright/test';
import { TestHelpers } from './fixtures/test-helpers';

/**
 * Authentication Flow Tests
 * 
 * Tests the complete user authentication flow including:
 * - Registration
 * - Login
 * - Logout
 * - Session persistence
 * - Error handling
 */

test.describe('Authentication', () => {
  let helpers: TestHelpers;

  test.beforeEach(async ({ page }) => {
    helpers = new TestHelpers(page);
  });

  test('should show login page by default', async ({ page }) => {
    await page.goto('/');
    
    // Should redirect to login
    await page.waitForURL('/login');
    
    // Check login form elements
    await expect(page.locator('input#email')).toBeVisible();
    await expect(page.locator('input#password')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('should register a new user', async ({ page }) => {
    const timestamp = Date.now();
    const email = `test${timestamp}@example.com`;
    
    await page.goto('/register');
    
    await page.fill('input[name="email"]', email);
    await page.fill('input[name="password"]', 'TestPassword123!');
    await page.fill('input[name="confirmPassword"]', 'TestPassword123!');
    
    await page.click('button[type="submit"]');
    
    // Should redirect to login after successful registration
    await page.waitForURL('/login');
    await helpers.waitForToast('Registration successful');
  });

  test('should login with valid credentials', async ({ page }) => {
    // Use test credentials from LOGIN_CREDENTIALS.md
    await helpers.login('demo@ethhook.com', 'Demo1234!');
    
    // Should be on dashboard
    await expect(page).toHaveURL('/dashboard');
    
    // Should see dashboard content
    await expect(page.locator('h1:has-text("Dashboard")')).toBeVisible();
    await expect(page.locator('text=Total Applications')).toBeVisible();
    await expect(page.locator('text=Active Endpoints')).toBeVisible();
  });

  test('should show error with invalid credentials', async ({ page }) => {
    await page.goto('/login');
    
    await page.fill('input#email', 'wrong@example.com');
    await page.fill('input#password', 'wrongpassword');
    
    await page.click('button[type="submit"]');
    
    // Should stay on login page
    await expect(page).toHaveURL('/login');
    
    // Should show error message
    await expect(page.locator('text=/Invalid.*password|Invalid credentials/i')).toBeVisible();
  });

  test('should logout successfully', async ({ page }) => {
    // Login first
    await helpers.login('demo@ethhook.com', 'Demo1234!');
    
    // Logout
    await helpers.logout();
    
    // Should be redirected to login
    await expect(page).toHaveURL('/login');
    
    // Try to access protected route
    await page.goto('/dashboard');
    
    // Should be redirected back to login
    await page.waitForURL('/login');
  });

  test('should handle network errors gracefully', async ({ page }) => {
    // Simulate network failure
    await page.route('**/api/auth/**', route => route.abort());
    
    await page.goto('/login');
    await page.fill('input#email', 'test@example.com');
    await page.fill('input#password', 'password123');
    
    await page.click('button[type="submit"]');
    
    // Should show user-friendly error (not crash!)
    // This catches the "Failed to fetch" bug the user encountered
    const errorMessage = page.locator('text=/Network error|Failed to fetch|Unable to connect/i');
    await expect(errorMessage).toBeVisible({ timeout: 5000 });
  });

  test('should persist session after page refresh', async ({ page, context }) => {
    // Login
    await helpers.login('demo@ethhook.com', 'Demo1234!');
    
    // Get cookies
    const cookies = await context.cookies();
    expect(cookies.length).toBeGreaterThan(0);
    
    // Refresh page
    await page.reload();
    
    // Should still be logged in
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator('text=Dashboard')).toBeVisible();
  });

  test('should validate email format', async ({ page }) => {
    await page.goto('/login');
    
    await page.fill('input#email', 'invalid-email');
    await page.fill('input#password', 'password123');
    
    await page.click('button[type="submit"]');
    
    // Should show validation error or browser validation prevents submit
    // HTML5 email input type handles validation
    const emailInput = page.locator('input#email');
    const isInvalid = await emailInput.evaluate((el: HTMLInputElement) => !el.validity.valid);
    expect(isInvalid).toBeTruthy();
  });

  test('should validate password requirements', async ({ page }) => {
    await page.goto('/register');
    
    await page.fill('input[name="email"]', 'test@example.com');
    await page.fill('input[name="password"]', '123'); // Too short
    await page.fill('input[name="confirmPassword"]', '123');
    
    await page.click('button[type="submit"]');
    
    // Should show password validation error
    await expect(page.locator('text=/Password must be|at least|characters/i')).toBeVisible();
  });
});
