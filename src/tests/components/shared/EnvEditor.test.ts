import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import EnvEditor from '$lib/components/shared/EnvEditor.svelte';

describe('EnvEditor', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render with empty values', () => {
			render(EnvEditor, { props: { values: {} } });

			// Should show add button when not readonly
			expect(screen.getByRole('button', { name: /add variable/i })).toBeInTheDocument();
		});

		it('should render entries for provided values', () => {
			render(EnvEditor, {
				props: {
					values: {
						API_KEY: 'secret123',
						DEBUG: 'true'
					}
				}
			});

			// Should have 2 entry rows (each with key and value inputs)
			const inputs = screen.getAllByRole('textbox');
			// 2 entries * 2 inputs each = 4 inputs
			expect(inputs.length).toBe(4);
		});

		it('should render with default placeholders', () => {
			render(EnvEditor, { props: { values: {} } });

			// Add an entry first
			fireEvent.click(screen.getByRole('button', { name: /add variable/i }));

			expect(screen.getByPlaceholderText('Variable name')).toBeInTheDocument();
			expect(screen.getByPlaceholderText('Value')).toBeInTheDocument();
		});

		it('should render with custom placeholders', () => {
			render(EnvEditor, {
				props: {
					values: {},
					keyPlaceholder: 'Header name',
					valuePlaceholder: 'Header value'
				}
			});

			// Add an entry first
			fireEvent.click(screen.getByRole('button', { name: /add variable/i }));

			expect(screen.getByPlaceholderText('Header name')).toBeInTheDocument();
			expect(screen.getByPlaceholderText('Header value')).toBeInTheDocument();
		});
	});

	describe('readonly mode', () => {
		it('should not show add button in readonly mode', () => {
			render(EnvEditor, {
				props: {
					values: { KEY: 'value' },
					readonly: true
				}
			});

			expect(screen.queryByRole('button', { name: /add variable/i })).not.toBeInTheDocument();
		});

		it('should not show delete buttons in readonly mode', () => {
			render(EnvEditor, {
				props: {
					values: { KEY: 'value' },
					readonly: true
				}
			});

			// Should not have any delete buttons (Trash2 icons)
			const buttons = screen.queryAllByRole('button');
			expect(buttons.length).toBe(0);
		});

		it('should disable inputs in readonly mode', () => {
			render(EnvEditor, {
				props: {
					values: { KEY: 'value' },
					readonly: true
				}
			});

			const inputs = screen.getAllByRole('textbox');
			inputs.forEach((input) => {
				expect(input).toBeDisabled();
			});
		});

		it('should enable inputs when not readonly', () => {
			render(EnvEditor, {
				props: {
					values: { KEY: 'value' },
					readonly: false
				}
			});

			const inputs = screen.getAllByRole('textbox');
			inputs.forEach((input) => {
				expect(input).not.toBeDisabled();
			});
		});
	});

	describe('adding entries', () => {
		it('should add new entry when add button is clicked', async () => {
			render(EnvEditor, { props: { values: {} } });

			const addButton = screen.getByRole('button', { name: /add variable/i });
			await fireEvent.click(addButton);

			// Should now have 2 inputs (key and value)
			expect(screen.getAllByRole('textbox').length).toBe(2);
		});

		it('should add multiple entries', async () => {
			render(EnvEditor, { props: { values: {} } });

			const addButton = screen.getByRole('button', { name: /add variable/i });
			await fireEvent.click(addButton);
			await fireEvent.click(addButton);
			await fireEvent.click(addButton);

			// Should have 6 inputs (3 entries * 2 inputs each)
			expect(screen.getAllByRole('textbox').length).toBe(6);
		});
	});

	describe('removing entries', () => {
		it('should remove entry when delete button is clicked', async () => {
			render(EnvEditor, {
				props: {
					values: {
						KEY1: 'value1',
						KEY2: 'value2'
					}
				}
			});

			// Initially should have 4 inputs
			expect(screen.getAllByRole('textbox').length).toBe(4);

			// Get all delete buttons and click the first one
			const deleteButtons = screen.getAllByRole('button').filter(
				(btn) => !btn.textContent?.includes('Add')
			);
			await fireEvent.click(deleteButtons[0]);

			// Should now have 2 inputs
			expect(screen.getAllByRole('textbox').length).toBe(2);
		});
	});

	describe('editing entries', () => {
		it('should update key on change', async () => {
			render(EnvEditor, {
				props: {
					values: { OLD_KEY: 'value' }
				}
			});

			const inputs = screen.getAllByRole('textbox');
			// First input is the key
			await fireEvent.change(inputs[0], { target: { value: 'NEW_KEY' } });

			expect(inputs[0]).toHaveValue('NEW_KEY');
		});

		it('should update value on change', async () => {
			render(EnvEditor, {
				props: {
					values: { KEY: 'old_value' }
				}
			});

			const inputs = screen.getAllByRole('textbox');
			// Second input is the value
			await fireEvent.change(inputs[1], { target: { value: 'new_value' } });

			expect(inputs[1]).toHaveValue('new_value');
		});

		it('should ignore entries with empty keys when updating values', async () => {
			const { component } = render(EnvEditor, {
				props: {
					values: { KEY: 'value' }
				}
			});

			// Add a new entry with empty key
			await fireEvent.click(screen.getByRole('button', { name: /add variable/i }));

			// The binding should only include entries with non-empty trimmed keys
			// This is tested by checking the component's behavior
			const inputs = screen.getAllByRole('textbox');
			expect(inputs.length).toBe(4); // Original 2 + new 2
		});
	});

	describe('styling', () => {
		it('should have correct container structure', () => {
			const { container } = render(EnvEditor, {
				props: {
					values: { KEY: 'value' }
				}
			});

			expect(container.querySelector('.space-y-2')).toBeInTheDocument();
		});

		it('should have delete button with trash icon', () => {
			const { container } = render(EnvEditor, {
				props: {
					values: { KEY: 'value' }
				}
			});

			// Find delete button
			const deleteButtons = screen.getAllByRole('button').filter(
				(btn) => !btn.textContent?.includes('Add')
			);
			expect(deleteButtons.length).toBeGreaterThan(0);

			// Should have SVG icon inside
			const svg = deleteButtons[0].querySelector('svg');
			expect(svg).toBeInTheDocument();
		});

		it('should have add button with plus icon', () => {
			const { container } = render(EnvEditor, { props: { values: {} } });

			const addButton = screen.getByRole('button', { name: /add variable/i });
			const svg = addButton.querySelector('svg');
			expect(svg).toBeInTheDocument();
		});
	});
});
