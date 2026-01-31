import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import CommandForm from '$lib/components/commands/CommandForm.svelte';

describe('CommandForm', () => {
	const mockOnSubmit = vi.fn();
	const mockOnCancel = vi.fn();

	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render import section', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByText('Import from Markdown')).toBeInTheDocument();
		});

		it('should render paste button', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Paste/i })).toBeInTheDocument();
		});

		it('should render file import button', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /File/i })).toBeInTheDocument();
		});

		it('should render name input', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Name/i)).toBeInTheDocument();
		});

		it('should render description input', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Description/i)).toBeInTheDocument();
		});

		it('should render allowed tools input', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Allowed Tools/i)).toBeInTheDocument();
		});

		it('should render argument hint input', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Argument Hint/i)).toBeInTheDocument();
		});

		it('should render model override select', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Model Override/i)).toBeInTheDocument();
		});

		it('should render command prompt textarea', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Command Prompt/i)).toBeInTheDocument();
		});

		it('should render tags input', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Tags/i)).toBeInTheDocument();
		});

		it('should render Cancel button', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Cancel/i })).toBeInTheDocument();
		});

		it('should render Create Command button for new command', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Create Command/i })).toBeInTheDocument();
		});

		it('should render Update Command button when editing', () => {
			render(CommandForm, {
				props: {
					initialValues: { name: 'test-command' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByRole('button', { name: /Update Command/i })).toBeInTheDocument();
		});
	});

	describe('validation', () => {
		it('should show error when name is empty', async () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Create Command/i }));

			expect(screen.getByText('Name is required')).toBeInTheDocument();
		});

		it('should show error when name has invalid characters', async () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'Invalid Name!' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Command/i }));

			expect(
				screen.getByText('Name must contain only lowercase letters, numbers, and hyphens')
			).toBeInTheDocument();
		});

		it('should show error when name contains reserved word', async () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'my-claude-command' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Command/i }));

			expect(screen.getByText('Name cannot contain reserved word "claude"')).toBeInTheDocument();
		});

		it('should show error when content is empty', async () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'valid-command' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Command/i }));

			expect(screen.getByText('Content is required')).toBeInTheDocument();
		});
	});

	describe('form submission', () => {
		it('should call onSubmit with correct values', async () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'my-command' }
			});
			await fireEvent.input(screen.getByLabelText(/Description/i), {
				target: { value: 'A test command' }
			});
			await fireEvent.input(screen.getByLabelText(/Command Prompt/i), {
				target: { value: '# Run this\n\nDo something.' }
			});
			await fireEvent.input(screen.getByLabelText(/Allowed Tools/i), {
				target: { value: 'Bash, Read' }
			});
			await fireEvent.input(screen.getByLabelText(/Argument Hint/i), {
				target: { value: '[file]' }
			});
			await fireEvent.input(screen.getByLabelText(/Tags/i), {
				target: { value: 'utility, dev' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Command/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith({
				name: 'my-command',
				description: 'A test command',
				content: '# Run this\n\nDo something.',
				allowedTools: ['Bash', 'Read'],
				argumentHint: '[file]',
				model: undefined,
				tags: ['utility', 'dev']
			});
		});

		it('should not call onSubmit when validation fails', async () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Create Command/i }));

			expect(mockOnSubmit).not.toHaveBeenCalled();
		});
	});

	describe('cancel', () => {
		it('should call onCancel when Cancel button clicked', async () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Cancel/i }));

			expect(mockOnCancel).toHaveBeenCalledOnce();
		});
	});

	describe('model selection', () => {
		it('should have model options', () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			const select = screen.getByLabelText(/Model Override/i);
			expect(select).toBeInTheDocument();
			expect(select.querySelector('option[value="opus"]')).toBeInTheDocument();
			expect(select.querySelector('option[value="sonnet"]')).toBeInTheDocument();
			expect(select.querySelector('option[value="haiku"]')).toBeInTheDocument();
		});

		it('should submit with selected model', async () => {
			render(CommandForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'my-command' }
			});
			await fireEvent.input(screen.getByLabelText(/Command Prompt/i), {
				target: { value: 'Content' }
			});
			await fireEvent.change(screen.getByLabelText(/Model Override/i), {
				target: { value: 'haiku' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Command/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith(
				expect.objectContaining({
					model: 'haiku'
				})
			);
		});
	});

	describe('initial values', () => {
		it('should populate fields with initial values', () => {
			render(CommandForm, {
				props: {
					initialValues: {
						name: 'existing-command',
						description: 'My command',
						content: '# Content',
						allowedTools: ['Bash', 'Write'],
						argumentHint: '[arg]',
						model: 'sonnet',
						tags: ['test']
					},
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByDisplayValue('existing-command')).toBeInTheDocument();
			expect(screen.getByDisplayValue('My command')).toBeInTheDocument();
			expect(screen.getByDisplayValue('# Content')).toBeInTheDocument();
			expect(screen.getByDisplayValue('Bash, Write')).toBeInTheDocument();
			expect(screen.getByDisplayValue('[arg]')).toBeInTheDocument();
			const modelSelect = screen.getByLabelText(/Model Override/i) as HTMLSelectElement;
			expect(modelSelect.value).toBe('sonnet');
			expect(screen.getByDisplayValue('test')).toBeInTheDocument();
		});
	});
});
