import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import SkillForm from '$lib/components/skills/SkillForm.svelte';

describe('SkillForm', () => {
	const mockOnSubmit = vi.fn();
	const mockOnCancel = vi.fn();

	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render import section', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByText('Import from Markdown')).toBeInTheDocument();
		});

		it('should render paste button', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Paste/i })).toBeInTheDocument();
		});

		it('should render file import button', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /File/i })).toBeInTheDocument();
		});

		it('should render name input', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Name/i)).toBeInTheDocument();
		});

		it('should render description input', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Description/i)).toBeInTheDocument();
		});

		it('should render allowed tools input', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Allowed Tools/i)).toBeInTheDocument();
		});

		it('should render model override select', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Model Override/i)).toBeInTheDocument();
		});

		it('should render disable model invocation checkbox', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Disable Model Invocation/i)).toBeInTheDocument();
		});

		it('should render skill instructions textarea', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Skill Instructions/i)).toBeInTheDocument();
		});

		it('should render tags input', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Tags/i)).toBeInTheDocument();
		});

		it('should render Cancel button', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Cancel/i })).toBeInTheDocument();
		});

		it('should render Create Skill button for new skill', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Create Skill/i })).toBeInTheDocument();
		});

		it('should render Update Skill button when editing', () => {
			render(SkillForm, {
				props: {
					initialValues: { name: 'test-skill' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByRole('button', { name: /Update Skill/i })).toBeInTheDocument();
		});
	});

	describe('validation', () => {
		it('should show error when name is empty', async () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Create Skill/i }));

			expect(screen.getByText('Name is required')).toBeInTheDocument();
		});

		it('should show error when name has invalid characters', async () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'Invalid Name!' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Skill/i }));

			expect(
				screen.getByText('Name must contain only lowercase letters, numbers, and hyphens')
			).toBeInTheDocument();
		});

		it('should show error when name contains reserved word', async () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'my-claude-skill' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Skill/i }));

			expect(screen.getByText('Name cannot contain reserved word "claude"')).toBeInTheDocument();
		});

		it('should show error when name contains anthropic', async () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'anthropic-skill' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Skill/i }));

			expect(
				screen.getByText('Name cannot contain reserved word "anthropic"')
			).toBeInTheDocument();
		});

		it('should show error when content is empty', async () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'valid-skill' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Skill/i }));

			expect(screen.getByText('Content is required')).toBeInTheDocument();
		});
	});

	describe('form submission', () => {
		it('should call onSubmit with correct values', async () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'my-skill' }
			});
			await fireEvent.input(screen.getByLabelText(/Description/i), {
				target: { value: 'A test skill' }
			});
			await fireEvent.input(screen.getByLabelText(/Skill Instructions/i), {
				target: { value: '# Instructions\n\nDo this.' }
			});
			await fireEvent.input(screen.getByLabelText(/Allowed Tools/i), {
				target: { value: 'Read, Edit' }
			});
			await fireEvent.input(screen.getByLabelText(/Tags/i), {
				target: { value: 'testing, dev' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Skill/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith({
				name: 'my-skill',
				description: 'A test skill',
				content: '# Instructions\n\nDo this.',
				allowedTools: ['Read', 'Edit'],
				model: undefined,
				disableModelInvocation: undefined,
				tags: ['testing', 'dev']
			});
		});

		it('should not call onSubmit when validation fails', async () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Create Skill/i }));

			expect(mockOnSubmit).not.toHaveBeenCalled();
		});
	});

	describe('cancel', () => {
		it('should call onCancel when Cancel button clicked', async () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Cancel/i }));

			expect(mockOnCancel).toHaveBeenCalledOnce();
		});
	});

	describe('model selection', () => {
		it('should have model options', () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			const select = screen.getByLabelText(/Model Override/i);
			expect(select).toBeInTheDocument();
			expect(select.querySelector('option[value="opus"]')).toBeInTheDocument();
			expect(select.querySelector('option[value="sonnet"]')).toBeInTheDocument();
			expect(select.querySelector('option[value="haiku"]')).toBeInTheDocument();
		});

		it('should submit with selected model', async () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'my-skill' }
			});
			await fireEvent.input(screen.getByLabelText(/Skill Instructions/i), {
				target: { value: 'Content' }
			});
			await fireEvent.change(screen.getByLabelText(/Model Override/i), {
				target: { value: 'opus' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Skill/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith(
				expect.objectContaining({
					model: 'opus'
				})
			);
		});
	});

	describe('disable model invocation', () => {
		it('should submit with disableModelInvocation when checked', async () => {
			render(SkillForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'my-skill' }
			});
			await fireEvent.input(screen.getByLabelText(/Skill Instructions/i), {
				target: { value: 'Content' }
			});
			await fireEvent.click(screen.getByLabelText(/Disable Model Invocation/i));

			await fireEvent.click(screen.getByRole('button', { name: /Create Skill/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith(
				expect.objectContaining({
					disableModelInvocation: true
				})
			);
		});
	});

	describe('initial values', () => {
		it('should populate fields with initial values', () => {
			render(SkillForm, {
				props: {
					initialValues: {
						name: 'existing-skill',
						description: 'My skill',
						content: '# Content',
						allowedTools: ['Read', 'Write'],
						model: 'sonnet',
						disableModelInvocation: true,
						tags: ['test']
					},
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByDisplayValue('existing-skill')).toBeInTheDocument();
			expect(screen.getByDisplayValue('My skill')).toBeInTheDocument();
			expect(screen.getByDisplayValue('# Content')).toBeInTheDocument();
			expect(screen.getByDisplayValue('Read, Write')).toBeInTheDocument();
			// Model is a select element
			const modelSelect = screen.getByLabelText(/Model Override/i) as HTMLSelectElement;
			expect(modelSelect.value).toBe('sonnet');
			expect(screen.getByLabelText(/Disable Model Invocation/i)).toBeChecked();
			expect(screen.getByDisplayValue('test')).toBeInTheDocument();
		});
	});
});
