import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import SubAgentForm from '$lib/components/subagents/SubAgentForm.svelte';

describe('SubAgentForm', () => {
	const mockOnSubmit = vi.fn();
	const mockOnCancel = vi.fn();

	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render import section', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByText('Import from Markdown')).toBeInTheDocument();
		});

		it('should render paste button', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Paste/i })).toBeInTheDocument();
		});

		it('should render file import button', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /File/i })).toBeInTheDocument();
		});

		it('should render name input', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Name/i)).toBeInTheDocument();
		});

		it('should render description textarea', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Description/i)).toBeInTheDocument();
		});

		it('should render model select', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Model/i)).toBeInTheDocument();
		});

		it('should render permission mode select', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Permission Mode/i)).toBeInTheDocument();
		});

		it('should render allowed tools input', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Allowed Tools/i)).toBeInTheDocument();
		});

		it('should render auto-load skills input', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Auto-load Skills/i)).toBeInTheDocument();
		});

		it('should render sub-agent prompt textarea', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Sub-Agent Prompt/i)).toBeInTheDocument();
		});

		it('should render tags input', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Tags/i)).toBeInTheDocument();
		});

		it('should render Cancel button', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Cancel/i })).toBeInTheDocument();
		});

		it('should render Create Sub-Agent button for new sub-agent', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Create Sub-Agent/i })).toBeInTheDocument();
		});

		it('should render Update Sub-Agent button when editing', () => {
			render(SubAgentForm, {
				props: {
					initialValues: { name: 'test-agent' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByRole('button', { name: /Update Sub-Agent/i })).toBeInTheDocument();
		});
	});

	describe('validation', () => {
		it('should show error when name is empty', async () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Create Sub-Agent/i }));

			expect(screen.getByText('Name is required')).toBeInTheDocument();
		});

		it('should show error when name has invalid format', async () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: '1invalid' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Sub-Agent/i }));

			expect(
				screen.getByText(
					'Name must start with a lowercase letter and contain only lowercase letters, numbers, and hyphens'
				)
			).toBeInTheDocument();
		});

		it('should show error when description is empty', async () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'valid-agent' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Sub-Agent/i }));

			expect(screen.getByText('Description is required')).toBeInTheDocument();
		});

		it('should show error when content is empty', async () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'valid-agent' }
			});
			await fireEvent.input(screen.getByLabelText(/Description/i), {
				target: { value: 'A description' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Sub-Agent/i }));

			expect(screen.getByText('Content is required')).toBeInTheDocument();
		});
	});

	describe('form submission', () => {
		it('should call onSubmit with correct values', async () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'my-agent' }
			});
			await fireEvent.input(screen.getByLabelText(/Description/i), {
				target: { value: 'A test agent' }
			});
			await fireEvent.input(screen.getByLabelText(/Sub-Agent Prompt/i), {
				target: { value: 'You are a specialized agent.' }
			});
			await fireEvent.input(screen.getByLabelText(/Allowed Tools/i), {
				target: { value: 'Read, Edit' }
			});
			await fireEvent.input(screen.getByLabelText(/Auto-load Skills/i), {
				target: { value: 'commit, review' }
			});
			await fireEvent.input(screen.getByLabelText(/Tags/i), {
				target: { value: 'testing, dev' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Sub-Agent/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith({
				name: 'my-agent',
				description: 'A test agent',
				content: 'You are a specialized agent.',
				model: undefined,
				permissionMode: undefined,
				tools: ['Read', 'Edit'],
				skills: ['commit', 'review'],
				tags: ['testing', 'dev']
			});
		});

		it('should not call onSubmit when validation fails', async () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Create Sub-Agent/i }));

			expect(mockOnSubmit).not.toHaveBeenCalled();
		});
	});

	describe('cancel', () => {
		it('should call onCancel when Cancel button clicked', async () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Cancel/i }));

			expect(mockOnCancel).toHaveBeenCalledOnce();
		});
	});

	describe('model selection', () => {
		it('should have model options', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			const select = screen.getByLabelText(/Model/i);
			expect(select).toBeInTheDocument();
			expect(select.querySelector('option[value="sonnet"]')).toBeInTheDocument();
			expect(select.querySelector('option[value="opus"]')).toBeInTheDocument();
			expect(select.querySelector('option[value="haiku"]')).toBeInTheDocument();
		});

		it('should submit with selected model', async () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'my-agent' }
			});
			await fireEvent.input(screen.getByLabelText(/Description/i), {
				target: { value: 'A description' }
			});
			await fireEvent.input(screen.getByLabelText(/Sub-Agent Prompt/i), {
				target: { value: 'Content' }
			});
			await fireEvent.change(screen.getByLabelText(/Model/i), {
				target: { value: 'opus' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Sub-Agent/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith(
				expect.objectContaining({
					model: 'opus'
				})
			);
		});
	});

	describe('permission mode selection', () => {
		it('should have permission mode options', () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			const select = screen.getByLabelText(/Permission Mode/i);
			expect(select).toBeInTheDocument();
			expect(select.querySelector('option[value="acceptEdits"]')).toBeInTheDocument();
			expect(select.querySelector('option[value="dontAsk"]')).toBeInTheDocument();
			expect(select.querySelector('option[value="plan"]')).toBeInTheDocument();
		});

		it('should submit with selected permission mode', async () => {
			render(SubAgentForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'my-agent' }
			});
			await fireEvent.input(screen.getByLabelText(/Description/i), {
				target: { value: 'A description' }
			});
			await fireEvent.input(screen.getByLabelText(/Sub-Agent Prompt/i), {
				target: { value: 'Content' }
			});
			await fireEvent.change(screen.getByLabelText(/Permission Mode/i), {
				target: { value: 'acceptEdits' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Sub-Agent/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith(
				expect.objectContaining({
					permissionMode: 'acceptEdits'
				})
			);
		});
	});

	describe('initial values', () => {
		it('should populate fields with initial values', () => {
			render(SubAgentForm, {
				props: {
					initialValues: {
						name: 'existing-agent',
						description: 'My agent',
						content: '# Content',
						model: 'sonnet',
						permissionMode: 'acceptEdits',
						tools: ['Bash', 'Read'],
						skills: ['commit'],
						tags: ['test']
					},
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByDisplayValue('existing-agent')).toBeInTheDocument();
			expect(screen.getByDisplayValue('My agent')).toBeInTheDocument();
			expect(screen.getByDisplayValue('# Content')).toBeInTheDocument();
			const modelSelect = screen.getByLabelText(/Model/i) as HTMLSelectElement;
			expect(modelSelect.value).toBe('sonnet');
			const permissionSelect = screen.getByLabelText(/Permission Mode/i) as HTMLSelectElement;
			expect(permissionSelect.value).toBe('acceptEdits');
			expect(screen.getByDisplayValue('Bash, Read')).toBeInTheDocument();
			expect(screen.getByDisplayValue('commit')).toBeInTheDocument();
			expect(screen.getByDisplayValue('test')).toBeInTheDocument();
		});
	});
});
