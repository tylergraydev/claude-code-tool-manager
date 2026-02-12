import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import EnvEditor from '$lib/components/shared/EnvEditor.svelte';

describe('EnvEditor Component', () => {
	it('should render existing key-value pairs', () => {
		render(EnvEditor, {
			props: { values: { API_KEY: 'secret', NODE_ENV: 'test' } }
		});

		const inputs = screen.getAllByRole('textbox');
		// Each pair has 2 inputs (key + value), so 4 total
		expect(inputs).toHaveLength(4);
	});

	it('should render add button when not readonly', () => {
		render(EnvEditor, { props: { values: {} } });

		expect(screen.getByText('Add variable')).toBeInTheDocument();
	});

	it('should not render add button when readonly', () => {
		render(EnvEditor, { props: { values: {}, readonly: true } });

		expect(screen.queryByText('Add variable')).not.toBeInTheDocument();
	});

	it('should add entry when add button clicked', async () => {
		render(EnvEditor, { props: { values: {} } });

		const addButton = screen.getByText('Add variable');
		await fireEvent.click(addButton);

		// Should now have 2 inputs (key + value for new entry)
		const inputs = screen.getAllByRole('textbox');
		expect(inputs).toHaveLength(2);
	});

	it('should render custom placeholders', () => {
		render(EnvEditor, {
			props: {
				values: {},
				keyPlaceholder: 'Header name',
				valuePlaceholder: 'Header value'
			}
		});

		// Click add to get inputs
		const addButton = screen.getByText('Add variable');
		fireEvent.click(addButton);

		expect(screen.getByPlaceholderText('Header name')).toBeInTheDocument();
		expect(screen.getByPlaceholderText('Header value')).toBeInTheDocument();
	});

	it('should show remove buttons when not readonly', () => {
		render(EnvEditor, {
			props: { values: { KEY: 'value' } }
		});

		const removeButtons = screen.getAllByRole('button').filter(
			(btn) => !btn.textContent?.includes('Add')
		);
		expect(removeButtons.length).toBeGreaterThan(0);
	});

	it('should not show remove buttons when readonly', () => {
		render(EnvEditor, {
			props: { values: { KEY: 'value' }, readonly: true }
		});

		const buttons = screen.queryAllByRole('button');
		expect(buttons).toHaveLength(0);
	});

	it('should render empty state with no entries', () => {
		render(EnvEditor, { props: { values: {} } });

		// Only the add button should be present, no key-value inputs
		const inputs = screen.queryAllByRole('textbox');
		expect(inputs).toHaveLength(0);
	});
});
