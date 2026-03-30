import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/types', async (importOriginal) => {
	const actual = await importOriginal() as any;
	return {
		...actual,
		CLAUDE_MODELS: actual.CLAUDE_MODELS ?? [
			{ value: 'claude-sonnet-4-5-20250929', label: 'Claude Sonnet 4.5', description: 'Balanced' }
		]
	};
});

describe('ModelOverridesEditor Component', () => {
	const mockSettings = {
		scope: 'user',
		availableModels: []
	};

	it('should render Model Overrides heading', async () => {
		const { default: ModelOverridesEditor } = await import('$lib/components/claude-settings/ModelOverridesEditor.svelte');
		render(ModelOverridesEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Model Overrides')).toBeInTheDocument();
	});

	it('should render Add Override button', async () => {
		const { default: ModelOverridesEditor } = await import('$lib/components/claude-settings/ModelOverridesEditor.svelte');
		render(ModelOverridesEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Add Override')).toBeInTheDocument();
	});

	it('should render existing overrides', async () => {
		const { default: ModelOverridesEditor } = await import('$lib/components/claude-settings/ModelOverridesEditor.svelte');
		const settingsWithOverrides = {
			...mockSettings,
			modelOverrides: { 'claude-sonnet-4-5-20250929': 'custom-sonnet-bedrock' }
		};
		const { container } = render(ModelOverridesEditor, {
			props: { settings: settingsWithOverrides as any, onsave: vi.fn() }
		});
		const inputs = container.querySelectorAll('input[type="text"]') as NodeListOf<HTMLInputElement>;
		const values = Array.from(inputs).map(i => i.value);
		expect(values).toContain('claude-sonnet-4-5-20250929');
		expect(values).toContain('custom-sonnet-bedrock');
	});

	it('should add a new override row when Add Override clicked', async () => {
		const { default: ModelOverridesEditor } = await import('$lib/components/claude-settings/ModelOverridesEditor.svelte');
		const { container } = render(ModelOverridesEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		const inputsBefore = container.querySelectorAll('input[type="text"]');
		expect(inputsBefore.length).toBe(0);
		await fireEvent.click(screen.getByText('Add Override'));
		const inputsAfter = container.querySelectorAll('input[type="text"]');
		expect(inputsAfter.length).toBe(2); // key + value
	});

	it('should render remove button for each override', async () => {
		const { default: ModelOverridesEditor } = await import('$lib/components/claude-settings/ModelOverridesEditor.svelte');
		const settingsWithOverrides = {
			...mockSettings,
			modelOverrides: { 'claude-sonnet-4-5-20250929': 'custom-sonnet' }
		};
		const { container } = render(ModelOverridesEditor, {
			props: { settings: settingsWithOverrides as any, onsave: vi.fn() }
		});
		const removeButtons = container.querySelectorAll('button[title="Remove override"]');
		expect(removeButtons.length).toBe(1);
	});

	it('should remove an override when remove button clicked', async () => {
		const { default: ModelOverridesEditor } = await import('$lib/components/claude-settings/ModelOverridesEditor.svelte');
		const settingsWithOverrides = {
			...mockSettings,
			modelOverrides: { 'claude-sonnet-4-5-20250929': 'custom-sonnet' }
		};
		const { container } = render(ModelOverridesEditor, {
			props: { settings: settingsWithOverrides as any, onsave: vi.fn() }
		});
		const removeButton = container.querySelector('button[title="Remove override"]')!;
		await fireEvent.click(removeButton);
		const inputsAfter = container.querySelectorAll('input[type="text"]');
		expect(inputsAfter.length).toBe(0);
	});

	it('should save overrides as Record when values present', async () => {
		const { default: ModelOverridesEditor } = await import('$lib/components/claude-settings/ModelOverridesEditor.svelte');
		const onsave = vi.fn();
		const settingsWithOverrides = {
			...mockSettings,
			modelOverrides: { 'claude-sonnet-4-5-20250929': 'custom-sonnet' }
		};
		render(ModelOverridesEditor, {
			props: { settings: settingsWithOverrides as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Model Overrides'));
		const saved = onsave.mock.calls[0][0];
		expect(saved.modelOverrides).toEqual({ 'claude-sonnet-4-5-20250929': 'custom-sonnet' });
	});

	it('should render multiple overrides', async () => {
		const { default: ModelOverridesEditor } = await import('$lib/components/claude-settings/ModelOverridesEditor.svelte');
		const settingsWithMultiple = {
			...mockSettings,
			modelOverrides: {
				'claude-sonnet-4-5-20250929': 'bedrock-sonnet',
				'claude-opus-4-6': 'bedrock-opus'
			}
		};
		const { container } = render(ModelOverridesEditor, {
			props: { settings: settingsWithMultiple as any, onsave: vi.fn() }
		});
		const inputs = container.querySelectorAll('input[type="text"]') as NodeListOf<HTMLInputElement>;
		expect(inputs.length).toBe(4); // 2 pairs
	});

	it('should render column headers when overrides exist', async () => {
		const { default: ModelOverridesEditor } = await import('$lib/components/claude-settings/ModelOverridesEditor.svelte');
		const settingsWithOverrides = {
			...mockSettings,
			modelOverrides: { 'claude-sonnet-4-5-20250929': 'custom' }
		};
		render(ModelOverridesEditor, {
			props: { settings: settingsWithOverrides as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Anthropic Model ID')).toBeInTheDocument();
		expect(screen.getByText('Provider Model ID')).toBeInTheDocument();
	});

	it('should call onsave with undefined when no overrides', async () => {
		const { default: ModelOverridesEditor } = await import('$lib/components/claude-settings/ModelOverridesEditor.svelte');
		const onsave = vi.fn();
		render(ModelOverridesEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Model Overrides'));
		const saved = onsave.mock.calls[0][0];
		expect(saved.modelOverrides).toBeUndefined();
	});

	it('should export ModelOverridesEditor from index', async () => {
		const exports = await import('$lib/components/claude-settings');
		expect(exports.ModelOverridesEditor).toBeDefined();
	});
});
