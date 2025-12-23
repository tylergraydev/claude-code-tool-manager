import { test as base, expect } from '@playwright/test';
import { mockTauriApi, getInvokeCalls, clearInvokeCalls } from './tauri-mock';

/**
 * Extended test fixture with Tauri mocking
 */
export const test = base.extend({
	page: async ({ page }, use) => {
		// Inject Tauri mocks before navigating
		await mockTauriApi(page);

		// Navigate to the app
		await page.goto('/', { waitUntil: 'networkidle' });

		// Wait for the app to be ready - check for sidebar
		await page.waitForSelector('aside', { timeout: 15000 });

		// Wait for navigation links to be present
		await page.waitForSelector('aside a[href="/library"]', { timeout: 10000 });

		// Wait for data to load (look for any content in the main area)
		await page.waitForTimeout(1000);

		// Use the page in the test
		await use(page);
	}
});

/**
 * Helper functions for E2E tests
 */
export const helpers = {
	/**
	 * Navigate to a specific route via sidebar
	 */
	async navigateTo(page: ReturnType<typeof test.page>, routeName: string) {
		const sidebarLink = page.locator(`nav a[href="/${routeName}"]`);
		await sidebarLink.click();
		await page.waitForURL(`**/${routeName}`);
	},

	/**
	 * Wait for data to load (loading spinner to disappear)
	 */
	async waitForDataLoad(page: ReturnType<typeof test.page>, timeout = 5000) {
		await page.waitForFunction(
			() => !document.querySelector('[data-testid="loading"]'),
			{ timeout }
		);
	},

	/**
	 * Get all invoke calls made during the test
	 */
	async getInvokeCalls(page: ReturnType<typeof test.page>) {
		return getInvokeCalls(page);
	},

	/**
	 * Clear invoke call log
	 */
	async clearInvokeCalls(page: ReturnType<typeof test.page>) {
		return clearInvokeCalls(page);
	},

	/**
	 * Check if a specific invoke command was called
	 */
	async wasInvoked(page: ReturnType<typeof test.page>, command: string) {
		const calls = await getInvokeCalls(page);
		return calls.some(call => call.cmd === command);
	},

	/**
	 * Get the arguments of a specific invoke call
	 */
	async getInvokeArgs(page: ReturnType<typeof test.page>, command: string) {
		const calls = await getInvokeCalls(page);
		const call = calls.find(c => c.cmd === command);
		return call?.args;
	}
};

export { expect };
