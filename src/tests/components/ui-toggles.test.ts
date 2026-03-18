import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/types', async (importOriginal) => {
	const actual = await importOriginal() as any;
	return {
		...actual,
		UI_TOGGLE_FIELDS: actual.UI_TOGGLE_FIELDS ?? [
			{ key: 'showTurnDuration', label: 'Show Turn Duration', description: 'Show how long each turn takes', defaultValue: false },
			{ key: 'spinnerTipsEnabled', label: 'Spinner Tips', description: 'Show tips while waiting', defaultValue: true }
		]
	};
});

describe('UITogglesEditor Component', () => {
	const mockSettings = {
		scope: 'project',
		availableModels: [],
		showTurnDuration: true,
		spinnerTipsEnabled: undefined
	};

	it('should render heading', async () => {
		const { default: UITogglesEditor } = await import('$lib/components/ui-toggles/UITogglesEditor.svelte');
		render(UITogglesEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('UI Toggles')).toBeInTheDocument();
	});

	it('should render save button', async () => {
		const { default: UITogglesEditor } = await import('$lib/components/ui-toggles/UITogglesEditor.svelte');
		render(UITogglesEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Save UI Toggle Settings')).toBeInTheDocument();
	});

	it('should call onsave when save clicked', async () => {
		const { default: UITogglesEditor } = await import('$lib/components/ui-toggles/UITogglesEditor.svelte');
		const onsave = vi.fn();
		render(UITogglesEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save UI Toggle Settings'));
		expect(onsave).toHaveBeenCalledOnce();
	});
});

describe('UI-toggles index.ts exports', () => {
	it('should export UITogglesEditor', async () => {
		const exports = await import('$lib/components/ui-toggles');
		expect(exports.UITogglesEditor).toBeDefined();
	});
});
