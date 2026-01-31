import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import CommandCard from '$lib/components/commands/CommandCard.svelte';
import type { Command } from '$lib/types';

describe('CommandCard', () => {
	const createMockCommand = (overrides: Partial<Command> = {}): Command => ({
		id: 1,
		name: 'test-command',
		instructions: 'Test instructions',
		source: 'user',
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01',
		...overrides
	});

	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render command name with slash prefix', () => {
			render(CommandCard, { props: { command: createMockCommand({ name: 'my-command' }) } });

			expect(screen.getByText('/my-command')).toBeInTheDocument();
		});

		it('should render command description when provided', () => {
			render(CommandCard, {
				props: { command: createMockCommand({ description: 'A helpful command' }) }
			});

			expect(screen.getByText('A helpful command')).toBeInTheDocument();
		});

		it('should not render description when not provided', () => {
			render(CommandCard, {
				props: { command: createMockCommand({ description: undefined }) }
			});

			expect(screen.queryByText(/helpful command/i)).not.toBeInTheDocument();
		});

		it('should have Terminal icon', () => {
			const { container } = render(CommandCard, { props: { command: createMockCommand() } });

			const iconContainer = container.querySelector('.bg-amber-100');
			expect(iconContainer).toBeInTheDocument();
		});
	});

	describe('source badges', () => {
		it('should show Auto badge for auto-detected commands', () => {
			render(CommandCard, {
				props: { command: createMockCommand({ source: 'auto-detected' }) }
			});

			expect(screen.getByText('Auto')).toBeInTheDocument();
		});

		it('should not show Auto badge for user commands', () => {
			render(CommandCard, {
				props: { command: createMockCommand({ source: 'user' }) }
			});

			expect(screen.queryByText('Auto')).not.toBeInTheDocument();
		});
	});

	describe('allowed tools badge', () => {
		it('should show tool count when tools are allowed', () => {
			render(CommandCard, {
				props: { command: createMockCommand({ allowedTools: ['tool1', 'tool2', 'tool3'] }) }
			});

			expect(screen.getByText('3 tools')).toBeInTheDocument();
		});

		it('should use singular form for 1 tool', () => {
			render(CommandCard, {
				props: { command: createMockCommand({ allowedTools: ['tool1'] }) }
			});

			expect(screen.getByText('1 tool')).toBeInTheDocument();
		});

		it('should not show tools badge when no tools allowed', () => {
			render(CommandCard, {
				props: { command: createMockCommand({ allowedTools: [] }) }
			});

			expect(screen.queryByText(/\d+ tools?/)).not.toBeInTheDocument();
		});
	});

	describe('argument hint badge', () => {
		it('should show argument hint when provided', () => {
			render(CommandCard, {
				props: { command: createMockCommand({ argumentHint: '<file>' }) }
			});

			expect(screen.getByText('<file>')).toBeInTheDocument();
		});

		it('should not show argument hint when not provided', () => {
			render(CommandCard, {
				props: { command: createMockCommand({ argumentHint: undefined }) }
			});

			expect(screen.queryByText(/<\w+>/)).not.toBeInTheDocument();
		});
	});

	describe('tags', () => {
		it('should show tags when provided', () => {
			render(CommandCard, {
				props: { command: createMockCommand({ tags: ['tag1', 'tag2'] }) }
			});

			expect(screen.getByText('tag1')).toBeInTheDocument();
			expect(screen.getByText('tag2')).toBeInTheDocument();
		});

		it('should show only first 2 tags', () => {
			render(CommandCard, {
				props: { command: createMockCommand({ tags: ['tag1', 'tag2', 'tag3', 'tag4'] }) }
			});

			expect(screen.getByText('tag1')).toBeInTheDocument();
			expect(screen.getByText('tag2')).toBeInTheDocument();
			expect(screen.queryByText('tag3')).not.toBeInTheDocument();
		});

		it('should show +N indicator for extra tags', () => {
			render(CommandCard, {
				props: { command: createMockCommand({ tags: ['tag1', 'tag2', 'tag3', 'tag4', 'tag5'] }) }
			});

			expect(screen.getByText('+3')).toBeInTheDocument();
		});
	});

	describe('actions menu', () => {
		it('should show menu button when showActions is true', () => {
			render(CommandCard, {
				props: { command: createMockCommand(), showActions: true }
			});

			const buttons = screen.getAllByRole('button');
			expect(buttons.length).toBeGreaterThan(0);
		});

		it('should not show menu button when showActions is false', () => {
			render(CommandCard, {
				props: { command: createMockCommand(), showActions: false }
			});

			expect(screen.queryByRole('button')).not.toBeInTheDocument();
		});

		it('should call onEdit when Edit is clicked', async () => {
			const onEdit = vi.fn();
			const command = createMockCommand();
			render(CommandCard, { props: { command, onEdit } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Edit'));

			expect(onEdit).toHaveBeenCalledWith(command);
		});

		it('should call onDelete when Delete is clicked', async () => {
			const onDelete = vi.fn();
			const command = createMockCommand();
			render(CommandCard, { props: { command, onDelete } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Delete'));

			expect(onDelete).toHaveBeenCalledWith(command);
		});

		it('should close menu after action', async () => {
			const onEdit = vi.fn();
			render(CommandCard, { props: { command: createMockCommand(), onEdit } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Edit'));

			expect(screen.queryByText('Edit')).not.toBeInTheDocument();
		});
	});
});
