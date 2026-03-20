import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	spinnerVerbLibrary: {
		verbs: [],
		isLoading: false,
		update: vi.fn(),
		reorder: vi.fn()
	}
}));

describe('SpinnerVerbForm Component', () => {
	it('should render verb input', async () => {
		const { default: SpinnerVerbForm } = await import('$lib/components/spinnerverbs/SpinnerVerbForm.svelte');
		render(SpinnerVerbForm, {
			props: {
				onSubmit: vi.fn(),
				onCancel: vi.fn()
			}
		});
		expect(screen.getByLabelText(/Verb/)).toBeInTheDocument();
	});

	it('should call onCancel when cancel clicked', async () => {
		const { default: SpinnerVerbForm } = await import('$lib/components/spinnerverbs/SpinnerVerbForm.svelte');
		const onCancel = vi.fn();
		render(SpinnerVerbForm, {
			props: { onSubmit: vi.fn(), onCancel }
		});
		await fireEvent.click(screen.getByText('Cancel'));
		expect(onCancel).toHaveBeenCalledOnce();
	});

	it('should populate initial values', async () => {
		const { default: SpinnerVerbForm } = await import('$lib/components/spinnerverbs/SpinnerVerbForm.svelte');
		render(SpinnerVerbForm, {
			props: {
				initialValues: { id: 1, verb: 'Pondering', isEnabled: true, sortOrder: 0 },
				onSubmit: vi.fn(),
				onCancel: vi.fn()
			}
		});
		const input = screen.getByLabelText(/Verb/) as HTMLInputElement;
		expect(input.value).toBe('Pondering');
	});
});

describe('SpinnerVerbList Component', () => {
	it('should show empty state when no verbs', async () => {
		const { default: SpinnerVerbList } = await import('$lib/components/spinnerverbs/SpinnerVerbList.svelte');
		render(SpinnerVerbList, {
			props: {
				onEdit: vi.fn(),
				onDelete: vi.fn()
			}
		});
		expect(screen.getByText(/No spinner verbs yet/)).toBeInTheDocument();
	});
});

describe('Spinnerverbs index.ts exports', () => {
	it('should export all components', async () => {
		const exports = await import('$lib/components/spinnerverbs');
		expect(exports.SpinnerVerbList).toBeDefined();
		expect(exports.SpinnerVerbForm).toBeDefined();
	});
});
