import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import HookCard from '$lib/components/hooks/HookCard.svelte';
import type { Hook } from '$lib/types';

describe('HookCard', () => {
	const createMockHook = (overrides: Partial<Hook> = {}): Hook => ({
		id: 1,
		name: 'Test Hook',
		eventType: 'Stop',
		hookType: 'command',
		command: 'echo done',
		source: 'user',
		isTemplate: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01',
		...overrides
	});

	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render hook name', () => {
			render(HookCard, { props: { hook: createMockHook({ name: 'My Custom Hook' }) } });

			expect(screen.getByText('My Custom Hook')).toBeInTheDocument();
		});

		it('should render hook description when provided', () => {
			render(HookCard, {
				props: { hook: createMockHook({ description: 'A test description' }) }
			});

			expect(screen.getByText('A test description')).toBeInTheDocument();
		});

		it('should not render description when not provided', () => {
			render(HookCard, {
				props: { hook: createMockHook({ description: undefined }) }
			});

			// Should not have description paragraph
			expect(screen.queryByText(/test description/i)).not.toBeInTheDocument();
		});
	});

	describe('event type badges', () => {
		it('should render PreToolUse event type', () => {
			render(HookCard, { props: { hook: createMockHook({ eventType: 'PreToolUse' }) } });

			expect(screen.getByText('PreToolUse')).toBeInTheDocument();
		});

		it('should render PostToolUse event type', () => {
			render(HookCard, { props: { hook: createMockHook({ eventType: 'PostToolUse' }) } });

			expect(screen.getByText('PostToolUse')).toBeInTheDocument();
		});

		it('should render Notification event type', () => {
			render(HookCard, { props: { hook: createMockHook({ eventType: 'Notification' }) } });

			expect(screen.getByText('Notification')).toBeInTheDocument();
		});

		it('should render Stop event type', () => {
			render(HookCard, { props: { hook: createMockHook({ eventType: 'Stop' }) } });

			expect(screen.getByText('Stop')).toBeInTheDocument();
		});

		it('should render SubagentStop event type', () => {
			render(HookCard, { props: { hook: createMockHook({ eventType: 'SubagentStop' }) } });

			expect(screen.getByText('SubagentStop')).toBeInTheDocument();
		});

		it('should apply PreToolUse colors', () => {
			const { container } = render(HookCard, {
				props: { hook: createMockHook({ eventType: 'PreToolUse' }) }
			});

			const badge = container.querySelector('.bg-blue-100');
			expect(badge).toBeInTheDocument();
		});

		it('should apply PostToolUse colors', () => {
			const { container } = render(HookCard, {
				props: { hook: createMockHook({ eventType: 'PostToolUse' }) }
			});

			const badge = container.querySelector('.bg-green-100');
			expect(badge).toBeInTheDocument();
		});

		it('should apply Stop colors', () => {
			const { container } = render(HookCard, {
				props: { hook: createMockHook({ eventType: 'Stop' }) }
			});

			const badge = container.querySelector('.bg-red-100');
			expect(badge).toBeInTheDocument();
		});
	});

	describe('hook type', () => {
		it('should show Command badge for command type', () => {
			render(HookCard, { props: { hook: createMockHook({ hookType: 'command' }) } });

			expect(screen.getByText('Command')).toBeInTheDocument();
		});

		it('should show Prompt badge for prompt type', () => {
			render(HookCard, { props: { hook: createMockHook({ hookType: 'prompt' }) } });

			expect(screen.getByText('Prompt')).toBeInTheDocument();
		});
	});

	describe('template badge', () => {
		it('should show Template badge when isTemplate is true', () => {
			render(HookCard, { props: { hook: createMockHook({ isTemplate: true }) } });

			expect(screen.getByText('Template')).toBeInTheDocument();
		});

		it('should not show Template badge when isTemplate is false', () => {
			render(HookCard, { props: { hook: createMockHook({ isTemplate: false }) } });

			expect(screen.queryByText('Template')).not.toBeInTheDocument();
		});
	});

	describe('matcher badge', () => {
		it('should show matcher when provided', () => {
			render(HookCard, { props: { hook: createMockHook({ matcher: 'test-pattern' }) } });

			expect(screen.getByText('test-pattern')).toBeInTheDocument();
		});

		it('should not show matcher when not provided', () => {
			render(HookCard, { props: { hook: createMockHook({ matcher: undefined }) } });

			// Only the main elements should be present
			expect(screen.queryByText(/pattern/i)).not.toBeInTheDocument();
		});
	});

	describe('timeout badge', () => {
		it('should show timeout when provided', () => {
			render(HookCard, { props: { hook: createMockHook({ timeout: 30 }) } });

			expect(screen.getByText('30s timeout')).toBeInTheDocument();
		});

		it('should not show timeout when not provided', () => {
			render(HookCard, { props: { hook: createMockHook({ timeout: undefined }) } });

			expect(screen.queryByText(/timeout/i)).not.toBeInTheDocument();
		});
	});

	describe('tags', () => {
		it('should show tags when provided', () => {
			render(HookCard, {
				props: { hook: createMockHook({ tags: ['tag1', 'tag2'] }) }
			});

			expect(screen.getByText('tag1')).toBeInTheDocument();
			expect(screen.getByText('tag2')).toBeInTheDocument();
		});

		it('should show only first 2 tags', () => {
			render(HookCard, {
				props: { hook: createMockHook({ tags: ['tag1', 'tag2', 'tag3', 'tag4'] }) }
			});

			expect(screen.getByText('tag1')).toBeInTheDocument();
			expect(screen.getByText('tag2')).toBeInTheDocument();
			expect(screen.queryByText('tag3')).not.toBeInTheDocument();
			expect(screen.queryByText('tag4')).not.toBeInTheDocument();
		});

		it('should show +N indicator for extra tags', () => {
			render(HookCard, {
				props: { hook: createMockHook({ tags: ['tag1', 'tag2', 'tag3', 'tag4'] }) }
			});

			expect(screen.getByText('+2')).toBeInTheDocument();
		});

		it('should not show tags section when empty', () => {
			render(HookCard, {
				props: { hook: createMockHook({ tags: [] }) }
			});

			// Should not find any tag badges
			expect(screen.queryByText(/^\+\d+$/)).not.toBeInTheDocument();
		});
	});

	describe('actions menu', () => {
		it('should show menu button when showActions is true', () => {
			render(HookCard, {
				props: { hook: createMockHook(), showActions: true }
			});

			const buttons = screen.getAllByRole('button');
			expect(buttons.length).toBeGreaterThan(0);
		});

		it('should not show menu button when showActions is false', () => {
			render(HookCard, {
				props: { hook: createMockHook(), showActions: false }
			});

			expect(screen.queryByRole('button')).not.toBeInTheDocument();
		});

		it('should toggle menu on button click', async () => {
			const onEdit = vi.fn();
			render(HookCard, {
				props: { hook: createMockHook(), onEdit }
			});

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);

			expect(screen.getByText('Edit')).toBeInTheDocument();
		});

		it('should call onEdit when Edit is clicked', async () => {
			const onEdit = vi.fn();
			const hook = createMockHook();
			render(HookCard, { props: { hook, onEdit } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Edit'));

			expect(onEdit).toHaveBeenCalledWith(hook);
		});

		it('should call onDelete when Delete is clicked', async () => {
			const onDelete = vi.fn();
			const hook = createMockHook();
			render(HookCard, { props: { hook, onDelete } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Delete'));

			expect(onDelete).toHaveBeenCalledWith(hook);
		});

		it('should call onDuplicate when Duplicate is clicked', async () => {
			const onDuplicate = vi.fn();
			const hook = createMockHook();
			render(HookCard, { props: { hook, onDuplicate } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Duplicate'));

			expect(onDuplicate).toHaveBeenCalledWith(hook);
		});

		it('should close menu after action', async () => {
			const onEdit = vi.fn();
			render(HookCard, { props: { hook: createMockHook(), onEdit } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Edit'));

			expect(screen.queryByText('Edit')).not.toBeInTheDocument();
		});
	});

	describe('styling', () => {
		it('should have Zap icon', () => {
			const { container } = render(HookCard, { props: { hook: createMockHook() } });

			const iconContainer = container.querySelector('.bg-orange-100');
			expect(iconContainer).toBeInTheDocument();
		});
	});
});
