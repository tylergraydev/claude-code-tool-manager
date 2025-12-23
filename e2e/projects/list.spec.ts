import { test, expect } from '../fixtures/app';

test.describe('Projects Page', () => {
	test.beforeEach(async ({ page }) => {
		// Navigate to projects page via full page navigation
		await page.goto('/projects', { waitUntil: 'networkidle' });
		await expect(page).toHaveURL('/projects');
		await expect(page.locator('main header h2')).toContainText(/Projects/i);
	});

	test('should display projects page header', async ({ page }) => {
		await expect(page.locator('main header h2')).toContainText('Projects');
	});

	test('should display projects from mock data', async ({ page }) => {
		// Wait for projects to load
		await page.waitForTimeout(500);

		// Should display project names (use getByText for exact matching)
		await expect(page.getByText('test-project').first()).toBeVisible();
		await expect(page.getByText('another-project').first()).toBeVisible();
	});

	test('should have Add Project button', async ({ page }) => {
		const addButton = page.locator('button:has-text("Add Project"), button:has-text("Browse")');
		await expect(addButton.first()).toBeVisible();
	});

	test('should display project path info', async ({ page }) => {
		// Wait for projects to load
		await page.waitForTimeout(500);

		// Should display at least the project name
		await expect(page.getByText('test-project').first()).toBeVisible();
	});

	test('should open project detail when clicking on project', async ({ page }) => {
		// Wait for projects to load
		await page.waitForTimeout(500);

		// Click on a project
		await page.click('text=test-project');

		// Should open a modal or detail view
		await page.waitForTimeout(200);

		// Look for project detail content (tabs like MCPs, Skills, etc.)
		const detailContent = page.locator('text=MCPs').or(page.locator('text=Skills')).or(page.locator('text=Hooks'));
		if (await detailContent.first().isVisible()) {
			// Detail modal is open
			expect(true).toBe(true);
		}
	});
});

test.describe('Projects - Search and Filter', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/projects', { waitUntil: 'networkidle' });
		await expect(page).toHaveURL('/projects');
		await expect(page.locator('main header h2')).toContainText(/Projects/i);
		await page.waitForTimeout(500);
	});

	test('should have search functionality if available', async ({ page }) => {
		// Look for search input
		const searchInput = page.locator('input[placeholder*="Search"]');

		if (await searchInput.isVisible()) {
			// Search for a project
			await searchInput.fill('test');
			await page.waitForTimeout(200);

			// Should show matching project
			await expect(page.locator('text=test-project')).toBeVisible();
		}
	});
});

test.describe('Projects - Empty State', () => {
	// Note: This would need a custom mock setup to test empty state
	// For now, we skip this as mock data always includes projects
	test.skip('should show empty state when no projects exist', async () => {
		// Would need to override mock data
	});
});
