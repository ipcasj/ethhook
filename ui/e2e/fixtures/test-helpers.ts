import { Page, expect } from '@playwright/test';

/**
 * Test Helper Class
 * 
 * Reusable functions for E2E tests to avoid code duplication
 * and make tests more maintainable.
 */
export class TestHelpers {
  constructor(private page: Page) {}

  /**
   * Login helper - authenticates a user
   */
  async login(email: string, password: string) {
    await this.page.goto('/login');
    await this.fillField('email-input', email);
    await this.fillField('password-input', password);
    await this.page.click('[data-testid="login-submit-button"]');
    
    // Wait for navigation to dashboard
    await this.page.waitForURL('/dashboard');
    
    // Verify we're logged in by checking for the heading specifically
    await expect(this.page.locator('h1:has-text("Dashboard")')).toBeVisible();
  }

  /**
   * Logout helper
   */
  async logout() {
    await this.page.click('button:has-text("Logout")');
    await this.page.waitForURL('/login');
  }

  /**
   * Navigate to a specific page via sidebar
   */
  async navigateTo(pageName: string) {
    await this.page.click(`nav a:has-text("${pageName}")`);
    await this.page.waitForLoadState('networkidle');
  }

  /**
   * Wait for toast notification
   */
  async waitForToast(message: string) {
    await expect(this.page.locator(`text=${message}`)).toBeVisible({ timeout: 5000 });
  }

  /**
   * Create a new application
   */
  async createApplication(name: string, description: string) {
    await this.navigateTo('Applications');
    await this.page.click('[data-testid="create-app-button"]');
    
    await this.fillField('app-name-input', name);
    await this.fillField('app-description-input', description);
    await this.page.click('button[type="submit"]');
    
    // Wait a moment for submission, then close dialog if still open
    await this.page.waitForTimeout(2000);
    
    // If dialog is still open, press Escape to close it
    const dialogStillOpen = await this.page.locator('[data-slot="dialog-overlay"]').isVisible();
    if (dialogStillOpen) {
      await this.page.keyboard.press('Escape');
      await this.page.waitForTimeout(500);
    }
  }

  /**
   * Create a new endpoint
   */
  async createEndpoint(appName: string, url: string, chainIds: number[]) {
    await this.navigateTo('Endpoints');
    await this.page.click('[data-testid="add-endpoint-button"]');
    
    // Select application from dropdown
    await this.page.selectOption('[data-testid="app-select"]', { label: appName });
    
    await this.fillField('webhook-url-input', url);
    
    // Select chain IDs (these are buttons)
    for (const chainId of chainIds) {
      const chainName = this.getChainName(chainId);
      await this.page.click(`button:has-text("${chainName}")`);
    }
    
    await this.page.click('button[type="submit"]');
    
    // Wait a moment for submission, then close dialog if still open
    await this.page.waitForTimeout(2000);
    
    // If dialog is still open, press Escape to close it
    const dialogStillOpen = await this.page.locator('[data-slot="dialog-overlay"]').isVisible();
    if (dialogStillOpen) {
      await this.page.keyboard.press('Escape');
      await this.page.waitForTimeout(500);
    }
  }

  /**
   * Get chain name from chain ID
   */
  private getChainName(chainId: number): string {
    const chains: Record<number, string> = {
      1: 'Ethereum Mainnet',
      11155111: 'Sepolia',
      137: 'Polygon',
      80001: 'Mumbai',
    };
    return chains[chainId] || 'Unknown';
  }

  /**
   * Wait for API response
   */
  async waitForApiResponse(url: string) {
    return await this.page.waitForResponse(response => 
      response.url().includes(url) && response.status() === 200
    );
  }

  /**
   * Check if element exists (without throwing)
   */
  async elementExists(selector: string): Promise<boolean> {
    try {
      const element = await this.page.locator(selector).count();
      return element > 0;
    } catch {
      return false;
    }
  }

  /**
   * Fill a form field using data-testid (preferred) or fallback to other selectors
   */
  async fillField(testId: string, value: string) {
    await this.page.fill(`[data-testid="${testId}"]`, value);
  }

  /**
   * Fill a form and submit
   */
  async fillFormAndSubmit(fields: Record<string, string>, submitButtonText = 'Submit') {
    for (const [name, value] of Object.entries(fields)) {
      await this.fillField(name, value);
    }
    await this.page.click(`button:has-text("${submitButtonText}")`);
  }

  /**
   * Delete item from table (using trash icon)
   */
  async deleteFromTable(rowText: string) {
    const row = this.page.locator(`tr:has-text("${rowText}")`);
    await row.locator('button[aria-label="Delete"]').click();
    
    // Confirm deletion in modal
    await this.page.click('button:has-text("Delete")');
    await this.waitForToast('Deleted successfully');
  }

  /**
   * Wait for loading to complete
   */
  async waitForLoading() {
    await this.page.waitForLoadState('networkidle');
    
    // Wait for any loading spinners to disappear
    const spinner = this.page.locator('[role="status"]');
    if (await spinner.isVisible()) {
      await spinner.waitFor({ state: 'hidden', timeout: 10000 });
    }
  }

  /**
   * Take a screenshot with a name
   */
  async screenshot(name: string) {
    await this.page.screenshot({ 
      path: `test-results/screenshots/${name}.png`,
      fullPage: true 
    });
  }
}
