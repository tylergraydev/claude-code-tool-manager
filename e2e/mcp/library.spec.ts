import { test, expect, helpers } from '../fixtures/app';

test.describe('MCP Library', () => {
	test.beforeEach(async ({ page }) => {
		// Navigate to library page via full page navigation
		await page.goto('/library', { waitUntil: 'networkidle' });
		await expect(page).toHaveURL('/library');
		// Wait for content to load
		await expect(page.locator('main header h2')).toContainText(/MCP Library/i);
	});

	test('should display library page header', async ({ page }) => {
		await expect(page.locator('main header h2')).toContainText('MCP Library');
	});

	test('should display MCPs from mock data', async ({ page }) => {
		// Wait for MCPs to load
		await page.waitForTimeout(500);

		// Check that mock MCP names are visible (use first() to handle multiple matches)
		await expect(page.getByText('filesystem').first()).toBeVisible();
		await expect(page.getByText('github').first()).toBeVisible();
		await expect(page.getByText('http-api').first()).toBeVisible();
	});

	test('should have Add MCP button', async ({ page }) => {
		const addButton = page.locator('button:has-text("Add MCP")');
		await expect(addButton).toBeVisible();
	});

	test('should open Add MCP modal when clicking Add button', async ({ page }) => {
		await page.click('button:has-text("Add MCP")');

		// Modal should appear
		await expect(page.getByText('Add New MCP').first()).toBeVisible();

		// Form name field should be visible
		await expect(page.locator('#name')).toBeVisible();
	});

	test('should close Add MCP modal when clicking Cancel', async ({ page }) => {
		// Open modal
		await page.click('button:has-text("Add MCP")');
		await expect(page.locator('text=Add New MCP')).toBeVisible();

		// Click cancel
		await page.click('button:has-text("Cancel")');

		// Modal should be gone
		await expect(page.locator('text=Add New MCP')).not.toBeVisible();
	});

	test('should have search functionality', async ({ page }) => {
		// Wait for MCPs to load
		await page.waitForTimeout(500);

		// Find search input
		const searchInput = page.locator('input[placeholder*="Search"]');
		await expect(searchInput).toBeVisible();

		// Search for a specific MCP
		await searchInput.fill('filesystem');
		await page.waitForTimeout(200);

		// Should show only matching MCP (use first() for specificity)
		await expect(page.getByText('filesystem').first()).toBeVisible();
		// github and http-api should be filtered out
		await expect(page.getByText('http-api')).not.toBeVisible();
	});

	test('should have type filter buttons', async ({ page }) => {
		// Check filter buttons exist
		await expect(page.locator('button:has-text("All")')).toBeVisible();
		await expect(page.locator('button:has-text("stdio")')).toBeVisible();
		await expect(page.locator('button:has-text("SSE")')).toBeVisible();
		await expect(page.locator('button:has-text("HTTP")')).toBeVisible();
	});

	test('should filter by type when clicking type filter', async ({ page }) => {
		// Wait for MCPs to load
		await page.waitForTimeout(500);

		// Click HTTP filter
		await page.click('button:has-text("HTTP")');
		await page.waitForTimeout(200);

		// Should show only HTTP MCPs
		await expect(page.locator('text=http-api')).toBeVisible();
		await expect(page.locator('text=filesystem')).not.toBeVisible();
		await expect(page.locator('text=github')).not.toBeVisible();
	});

	test('should filter by stdio type', async ({ page }) => {
		// Wait for MCPs to load
		await page.waitForTimeout(500);

		// Click stdio filter (use getByRole for specificity)
		await page.getByRole('button', { name: 'stdio' }).click();
		await page.waitForTimeout(200);

		// Should show only stdio MCPs (filesystem and github)
		await expect(page.getByText('filesystem').first()).toBeVisible();
		await expect(page.getByText('github').first()).toBeVisible();
		await expect(page.getByText('http-api')).not.toBeVisible();
	});

	test('should show all MCPs when clicking All filter', async ({ page }) => {
		// Wait for MCPs to load
		await page.waitForTimeout(500);

		// First filter by HTTP
		await page.getByRole('button', { name: 'HTTP' }).click();
		await page.waitForTimeout(200);

		// Then click All
		await page.getByRole('button', { name: 'All' }).click();
		await page.waitForTimeout(200);

		// All MCPs should be visible
		await expect(page.getByText('filesystem').first()).toBeVisible();
		await expect(page.getByText('github').first()).toBeVisible();
		await expect(page.getByText('http-api').first()).toBeVisible();
	});

	test('should display MCP counts in filter buttons', async ({ page }) => {
		// Wait for MCPs to load
		await page.waitForTimeout(500);

		// All button should show total count (3)
		const allButton = page.locator('button:has-text("All")');
		await expect(allButton).toContainText('3');

		// stdio button should show 2
		const stdioButton = page.locator('button:has-text("stdio")');
		await expect(stdioButton).toContainText('2');

		// HTTP button should show 1
		const httpButton = page.locator('button:has-text("HTTP")');
		await expect(httpButton).toContainText('1');
	});
});

test.describe('MCP Library - Empty State', () => {
	test('should show empty message when no MCPs match search', async ({ page }) => {
		await page.goto('/library', { waitUntil: 'networkidle' });
		await expect(page).toHaveURL('/library');
		await expect(page.locator('main header h2')).toContainText(/MCP Library/i);

		// Wait for MCPs to load
		await page.waitForTimeout(500);

		// Search for non-existent MCP
		const searchInput = page.locator('input[placeholder*="Search"]');
		await searchInput.fill('nonexistent-mcp-xyz');
		await page.waitForTimeout(200);

		// Should show "No matching MCPs" message
		await expect(page.locator('text=No matching MCPs')).toBeVisible();
	});
});
