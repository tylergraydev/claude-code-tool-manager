import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import HookForm from '$lib/components/hooks/HookForm.svelte';
import type { Hook } from '$lib/types';

// Mock HOOK_EVENT_TYPES from $lib/types
vi.mock('$lib/types', async () => {
	const actual = await vi.importActual('$lib/types');
	return {
		...actual,
		HOOK_EVENT_TYPES: [
			{
				value: 'PreToolUse',
				label: 'Pre Tool Use',
				description: 'Before a tool is used',
				matcherHint: 'Tool name'
			},
			{
				value: 'PostToolUse',
				label: 'Post Tool Use',
				description: 'After a tool is used',
				matcherHint: 'Tool name'
			},
			{
				value: 'SessionStart',
				label: 'Session Start',
				description: 'When session starts',
				matcherHint: null
			}
		]
	};
});

const mockTemplates: Hook[] = [
	{
		id: 1,
		name: 'Prettier Format',
		eventType: 'PostToolUse',
		hookType: 'command',
		command: 'npx prettier --write',
		source: 'user',
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	}
];

describe('HookForm', () => {
	const mockOnSubmit = vi.fn();
	const mockOnCancel = vi.fn();

	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render import section', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByText('Import from JSON or Template')).toBeInTheDocument();
		});

		it('should render paste button', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Paste/i })).toBeInTheDocument();
		});

		it('should render file import button', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /File/i })).toBeInTheDocument();
		});

		it('should render templates dropdown when templates provided', () => {
			render(HookForm, {
				props: { templates: mockTemplates, onSubmit: mockOnSubmit, onCancel: mockOnCancel }
			});

			expect(screen.getByText('Templates...')).toBeInTheDocument();
		});

		it('should NOT render templates dropdown when no templates', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.queryByText('Templates...')).not.toBeInTheDocument();
		});

		it('should render description input', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Description/i)).toBeInTheDocument();
		});

		it('should render event type selector', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByText('Event Type')).toBeInTheDocument();
			expect(screen.getByText('Pre Tool Use')).toBeInTheDocument();
			expect(screen.getByText('Post Tool Use')).toBeInTheDocument();
		});

		it('should render matcher pattern input', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Matcher Pattern/i)).toBeInTheDocument();
		});

		it('should render hook type toggle', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByText('Hook Type')).toBeInTheDocument();
			// Command appears as both hook type button and input label
			expect(screen.getAllByText('Command').length).toBeGreaterThanOrEqual(1);
			expect(screen.getByText('Prompt')).toBeInTheDocument();
		});

		it('should render command textarea by default', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Command/i)).toBeInTheDocument();
		});

		it('should render timeout input for command type', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Timeout/i)).toBeInTheDocument();
		});

		it('should render tags input', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Tags/i)).toBeInTheDocument();
		});

		it('should render Cancel button', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Cancel/i })).toBeInTheDocument();
		});

		it('should render Create Hook button for new hook', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Create Hook/i })).toBeInTheDocument();
		});

		it('should render Update Hook button when editing', () => {
			render(HookForm, {
				props: {
					initialValues: { name: 'test-hook' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByRole('button', { name: /Update Hook/i })).toBeInTheDocument();
		});
	});

	describe('validation', () => {
		it('should show error when command is empty for command type', async () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Create Hook/i }));

			expect(screen.getByText('Command is required for command hooks')).toBeInTheDocument();
			expect(mockOnSubmit).not.toHaveBeenCalled();
		});

		it('should show error when prompt is empty for prompt type', async () => {
			render(HookForm, {
				props: {
					initialValues: { hookType: 'prompt' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Hook/i }));

			expect(screen.getByText('Prompt is required for prompt hooks')).toBeInTheDocument();
		});

		it('should have timeout input with min value of 0', () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			const timeoutInput = screen.getByLabelText(/Timeout/i);
			expect(timeoutInput).toHaveAttribute('type', 'number');
			expect(timeoutInput).toHaveAttribute('min', '0');
		});
	});

	describe('form submission', () => {
		it('should call onSubmit with correct values for command type', async () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Description/i), {
				target: { value: 'Test description' }
			});
			await fireEvent.input(screen.getByLabelText(/Matcher Pattern/i), {
				target: { value: 'Write' }
			});
			await fireEvent.input(screen.getByLabelText(/Command/i), {
				target: { value: 'npx prettier' }
			});
			await fireEvent.input(screen.getByLabelText(/Timeout/i), {
				target: { value: '30' }
			});
			await fireEvent.input(screen.getByLabelText(/Tags/i), {
				target: { value: 'formatting, code' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Hook/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith({
				name: 'PostToolUse-Write',
				description: 'Test description',
				eventType: 'PostToolUse',
				matcher: 'Write',
				hookType: 'command',
				command: 'npx prettier',
				prompt: undefined,
				timeout: 30,
				tags: ['formatting', 'code']
			});
		});

		it('should call onSubmit with correct values for prompt type', async () => {
			render(HookForm, {
				props: {
					initialValues: { hookType: 'prompt' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			await fireEvent.input(screen.getByLabelText(/Prompt Text/i), {
				target: { value: 'Please review this code' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Hook/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith({
				name: 'PostToolUse',
				description: undefined,
				eventType: 'PostToolUse',
				matcher: undefined,
				hookType: 'prompt',
				command: undefined,
				prompt: 'Please review this code',
				timeout: undefined,
				tags: undefined
			});
		});

		it('should auto-generate name from event type when no matcher', async () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Command/i), {
				target: { value: 'test command' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Hook/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith(
				expect.objectContaining({
					name: 'PostToolUse'
				})
			);
		});
	});

	describe('cancel', () => {
		it('should call onCancel when Cancel button clicked', async () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Cancel/i }));

			expect(mockOnCancel).toHaveBeenCalledOnce();
		});
	});

	describe('hook type switching', () => {
		it('should show prompt textarea when prompt type is selected', async () => {
			render(HookForm, {
				props: {
					initialValues: { hookType: 'prompt' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByLabelText(/Prompt Text/i)).toBeInTheDocument();
			expect(screen.queryByLabelText(/Command/i)).not.toBeInTheDocument();
		});

		it('should hide timeout when prompt type is selected', () => {
			render(HookForm, {
				props: {
					initialValues: { hookType: 'prompt' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.queryByLabelText(/Timeout/i)).not.toBeInTheDocument();
		});
	});

	describe('event type selection', () => {
		it('should allow selecting different event types', async () => {
			render(HookForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByText('Pre Tool Use'));

			await fireEvent.input(screen.getByLabelText(/Command/i), {
				target: { value: 'test' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create Hook/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith(
				expect.objectContaining({
					eventType: 'PreToolUse'
				})
			);
		});
	});

	describe('initial values', () => {
		it('should populate fields with initial values', () => {
			render(HookForm, {
				props: {
					initialValues: {
						description: 'My hook',
						eventType: 'PreToolUse',
						matcher: 'Write',
						hookType: 'command',
						command: 'npm test',
						timeout: 60,
						tags: ['testing', 'ci']
					},
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByDisplayValue('My hook')).toBeInTheDocument();
			expect(screen.getByDisplayValue('Write')).toBeInTheDocument();
			expect(screen.getByDisplayValue('npm test')).toBeInTheDocument();
			expect(screen.getByDisplayValue('60')).toBeInTheDocument();
			expect(screen.getByDisplayValue('testing, ci')).toBeInTheDocument();
		});
	});
});
