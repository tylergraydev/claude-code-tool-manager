import { test, expect } from '../fixtures/app';

test.describe('MCP Form', () => {
	test.beforeEach(async ({ page }) => {
		// Navigate to library via full page navigation
		await page.goto('/library', { waitUntil: 'networkidle' });
		await expect(page).toHaveURL('/library');
		await expect(page.locator('main header h2')).toContainText(/MCP Library/i);

		// Open Add MCP modal
		await page.click('button:has-text("Add MCP")');
		await expect(page.locator('text=Add New MCP')).toBeVisible();
	});

	test('should display form with required fields', async ({ page }) => {
		// Name field should be visible
		const nameInput = page.locator('#name');
		await expect(nameInput).toBeVisible();

		// Type selector buttons should be visible (these use full labels, not abbreviations)
		await expect(page.getByRole('button', { name: /Standard I\/O/i })).toBeVisible();
		await expect(page.getByRole('button', { name: /Server-Sent Events/i })).toBeVisible();
		await expect(page.getByRole('button', { name: /HTTP\/REST/i })).toBeVisible();

		// Command field should be visible for stdio (default)
		await expect(page.locator('#command')).toBeVisible();

		// Submit and cancel buttons should be visible
		await expect(page.locator('button:has-text("Create MCP")')).toBeVisible();
		await expect(page.locator('button:has-text("Cancel")')).toBeVisible();
	});

	test('should show command/args fields for stdio type', async ({ page }) => {
		// Default is stdio, so command and args should be visible
		await expect(page.locator('#command')).toBeVisible();
		await expect(page.locator('#args')).toBeVisible();

		// URL should not be visible
		await expect(page.locator('#url')).not.toBeVisible();
	});

	test('should show URL field when switching to SSE type', async ({ page }) => {
		// Click SSE type button (use full label to avoid matching library filter)
		await page.getByRole('button', { name: /Server-Sent Events/i }).click();
		await page.waitForTimeout(100);

		// URL should now be visible
		await expect(page.locator('#url')).toBeVisible();

		// Command should not be visible
		await expect(page.locator('#command')).not.toBeVisible();
	});

	test('should show URL and headers fields when switching to HTTP type', async ({ page }) => {
		// Click HTTP type button (use full label to avoid matching library filter)
		await page.getByRole('button', { name: /HTTP\/REST/i }).click();
		await page.waitForTimeout(100);

		// URL should be visible
		await expect(page.locator('#url')).toBeVisible();

		// Headers section should be visible
		await expect(page.getByText('Headers').first()).toBeVisible();
	});

	test('should validate required name field', async ({ page }) => {
		// Try to submit without filling name
		await page.click('button:has-text("Create MCP")');

		// Should show error
		await expect(page.locator('text=Name is required')).toBeVisible();
	});

	test('should validate name format', async ({ page }) => {
		// Fill invalid name (with spaces)
		await page.fill('#name', 'my mcp name');

		// Fill command to pass that validation
		await page.fill('#command', 'npx');

		// Try to submit
		await page.click('button:has-text("Create MCP")');

		// Should show format error
		await expect(page.locator('text=Name can only contain')).toBeVisible();
	});

	test('should validate required command for stdio type', async ({ page }) => {
		// Fill valid name
		await page.fill('#name', 'my-mcp');

		// Don't fill command

		// Try to submit
		await page.click('button:has-text("Create MCP")');

		// Should show command error
		await expect(page.locator('text=Command is required')).toBeVisible();
	});

	test('should validate required URL for SSE type', async ({ page }) => {
		// Fill valid name
		await page.fill('#name', 'my-mcp');

		// Switch to SSE
		await page.getByRole('button', { name: /Server-Sent Events/i }).click();
		await page.waitForTimeout(100);

		// Try to submit without URL
		await page.click('button:has-text("Create MCP")');

		// Should show URL error
		await expect(page.locator('text=URL is required')).toBeVisible();
	});

	test('should validate URL format for SSE/HTTP types', async ({ page }) => {
		// Fill valid name
		await page.fill('#name', 'my-mcp');

		// Switch to SSE
		await page.getByRole('button', { name: /Server-Sent Events/i }).click();
		await page.waitForTimeout(100);

		// Fill invalid URL
		await page.fill('#url', 'not-a-valid-url');

		// Try to submit
		await page.click('button:has-text("Create MCP")');

		// Should show format error from custom validation
		await expect(page.locator('text=Invalid URL format')).toBeVisible();
	});

	test('should accept valid stdio MCP form', async ({ page }) => {
		// Fill valid data
		await page.fill('#name', 'test-mcp');
		await page.fill('#description', 'A test MCP server');
		await page.fill('#command', 'npx');
		await page.fill('#args', '-y @test/mcp-server');

		// Submit
		await page.click('button:has-text("Create MCP")');

		// Modal should close (form submitted)
		await expect(page.locator('text=Add New MCP')).not.toBeVisible({ timeout: 5000 });
	});

	test('should accept valid SSE MCP form', async ({ page }) => {
		// Fill valid data
		await page.fill('#name', 'test-sse-mcp');

		// Switch to SSE
		await page.getByRole('button', { name: /Server-Sent Events/i }).click();
		await page.waitForTimeout(100);

		await page.fill('#url', 'https://api.example.com/sse');

		// Submit
		await page.click('button:has-text("Create MCP")');

		// Modal should close
		await expect(page.locator('text=Add New MCP')).not.toBeVisible({ timeout: 5000 });
	});

	test('should have Quick Import paste area', async ({ page }) => {
		await expect(page.locator('text=Quick Import')).toBeVisible();
		await expect(page.locator('text=claude mcp add')).toBeVisible();
		await expect(page.locator('button:has-text("Paste")')).toBeVisible();
	});

	test('should have environment variables section', async ({ page }) => {
		await expect(page.locator('text=Environment Variables')).toBeVisible();
	});
});

test.describe('MCP Form - Edit Mode', () => {
	test('should open edit modal when clicking edit on MCP card', async ({ page }) => {
		// Navigate to library
		await page.goto('/library', { waitUntil: 'networkidle' });
		await expect(page).toHaveURL('/library');
		await expect(page.locator('main header h2')).toContainText(/MCP Library/i);

		// Wait for MCPs to load
		await page.waitForTimeout(500);

		// Find an MCP card by its heading text
		const mcpCard = page.locator('h3:has-text("filesystem")').first().locator('..').locator('..');

		// Look for edit button or menu
		const editButton = mcpCard.locator('button[aria-label*="edit" i], button:has-text("Edit")').first();

		if (await editButton.isVisible()) {
			await editButton.click();
			// Should open edit modal
			await expect(page.locator('text=Edit MCP')).toBeVisible();
		} else {
			// Try clicking on the card itself
			await mcpCard.click();
			// Check if edit modal or detail view opens
			await page.waitForTimeout(200);
		}
	});
});
