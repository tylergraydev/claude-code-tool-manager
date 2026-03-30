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
