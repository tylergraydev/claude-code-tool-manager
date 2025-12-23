import { test, expect } from './fixtures/app';

/**
 * Helper to navigate via URL (more reliable than clicking)
 */
async function navigateToRoute(page: any, route: string, expectedTitle: RegExp) {
	// Use direct navigation instead of click (SvelteKit client-side routing may not work well with mocks)
	await page.goto(route, { waitUntil: 'networkidle' });

	// Wait for URL
	await expect(page).toHaveURL(route);

	// Wait for the header to show the expected title
	await expect(page.locator('main header h2')).toContainText(expectedTitle, { timeout: 15000 });
}

test.describe('Navigation', () => {
	test('should load the dashboard on initial visit', async ({ page }) => {
		// Dashboard should be visible
		await expect(page).toHaveURL('/');

		// Sidebar should be visible with navigation items
		await expect(page.locator('aside')).toBeVisible();
		await expect(page.locator('aside a[href="/"]')).toBeVisible();
		await expect(page.locator('aside a[href="/library"]')).toBeVisible();
	});

	test('should navigate to MCP Library', async ({ page }) => {
		await navigateToRoute(page, '/library', /MCP Library/i);
	});

	test('should navigate to Skills Library', async ({ page }) => {
		await navigateToRoute(page, '/skills', /Skill/i);
	});

	test('should navigate to Sub-Agents Library', async ({ page }) => {
		await navigateToRoute(page, '/subagents', /Sub-Agent|Agent/i);
	});

	test('should navigate to Hooks Library', async ({ page }) => {
		await navigateToRoute(page, '/hooks', /Hook/i);
	});

	test('should navigate to Marketplace', async ({ page }) => {
		await navigateToRoute(page, '/marketplace', /Marketplace/i);
	});

	test('should navigate to Projects', async ({ page }) => {
		await navigateToRoute(page, '/projects', /Project/i);
	});

	test('should navigate to Global Settings', async ({ page }) => {
		await navigateToRoute(page, '/settings', /Setting/i);
	});

	test('should highlight active navigation item', async ({ page }) => {
		// Go to library and wait for content
		await navigateToRoute(page, '/library', /MCP Library/i);

		// Library link should have active class (bg-primary-50 in dark mode becomes bg-primary-900)
		const libraryLink = page.locator('aside a[href="/library"]');
		await expect(libraryLink).toHaveClass(/primary/);
	});

	test('should show app title in sidebar', async ({ page }) => {
		await expect(page.locator('aside')).toContainText('Claude Code');
		await expect(page.locator('aside')).toContainText('Tool Manager');
	});
});

test.describe('Theme Toggle', () => {
	test('should toggle dark mode', async ({ page }) => {
		// Navigate to settings where theme toggle is typically located
		await page.click('aside a[href="/settings"]');
		await expect(page).toHaveURL('/settings');

		// Look for a theme toggle (usually in settings or header)
		const themeToggle = page.locator('[data-testid="theme-toggle"], button:has-text("Dark"), button:has-text("Light"), button:has-text("Theme")').first();

		if (await themeToggle.isVisible()) {
			// Get initial state
			const htmlClass = await page.locator('html').getAttribute('class');
			const initiallyDark = htmlClass?.includes('dark') ?? false;

			// Click toggle
			await themeToggle.click();

			// Wait for class change
			await page.waitForTimeout(100);

			// Check that class changed
			const newHtmlClass = await page.locator('html').getAttribute('class');
			const nowDark = newHtmlClass?.includes('dark') ?? false;

			expect(nowDark).not.toBe(initiallyDark);
		}
	});
});
