import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import SubAgentCard from '$lib/components/subagents/SubAgentCard.svelte';
import type { SubAgent } from '$lib/types';

describe('SubAgentCard', () => {
	const createMockSubAgent = (overrides: Partial<SubAgent> = {}): SubAgent => ({
		id: 1,
		name: 'Test SubAgent',
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
		it('should render subagent name', () => {
			render(SubAgentCard, { props: { subagent: createMockSubAgent({ name: 'My Agent' }) } });

			expect(screen.getByText('My Agent')).toBeInTheDocument();
		});

		it('should render subagent description when provided', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent({ description: 'A helpful agent' }) }
			});

			expect(screen.getByText('A helpful agent')).toBeInTheDocument();
		});

		it('should not render description when not provided', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent({ description: undefined }) }
			});

			expect(screen.queryByText(/helpful agent/i)).not.toBeInTheDocument();
		});

		it('should have Bot icon', () => {
			const { container } = render(SubAgentCard, { props: { subagent: createMockSubAgent() } });

			const iconContainer = container.querySelector('.bg-indigo-100');
			expect(iconContainer).toBeInTheDocument();
		});
	});

	describe('source badges', () => {
		it('should show Auto badge for auto-detected subagents', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent({ source: 'auto-detected' }) }
			});

			expect(screen.getByText('Auto')).toBeInTheDocument();
		});

		it('should not show Auto badge for user subagents', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent({ source: 'user' }) }
			});

			expect(screen.queryByText('Auto')).not.toBeInTheDocument();
		});
	});

	describe('model badge', () => {
		it('should show model when provided', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent({ model: 'sonnet' }) }
			});

			expect(screen.getByText('sonnet')).toBeInTheDocument();
		});

		it('should not show model badge when not provided', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent({ model: undefined }) }
			});

			expect(screen.queryByText(/opus|sonnet|haiku/i)).not.toBeInTheDocument();
		});
	});

	describe('tools badge', () => {
		it('should show tool count when tools are provided', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent({ tools: ['tool1', 'tool2', 'tool3'] }) }
			});

			expect(screen.getByText('3 tools')).toBeInTheDocument();
		});

		it('should use singular form for 1 tool', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent({ tools: ['tool1'] }) }
			});

			expect(screen.getByText('1 tool')).toBeInTheDocument();
		});

		it('should not show tools badge when no tools', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent({ tools: [] }) }
			});

			expect(screen.queryByText(/\d+ tools?/)).not.toBeInTheDocument();
		});
	});

	describe('tags', () => {
		it('should show tags when provided', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent({ tags: ['tag1', 'tag2'] }) }
			});

			expect(screen.getByText('tag1')).toBeInTheDocument();
			expect(screen.getByText('tag2')).toBeInTheDocument();
		});

		it('should show only first 2 tags', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent({ tags: ['tag1', 'tag2', 'tag3', 'tag4'] }) }
			});

			expect(screen.getByText('tag1')).toBeInTheDocument();
			expect(screen.getByText('tag2')).toBeInTheDocument();
			expect(screen.queryByText('tag3')).not.toBeInTheDocument();
		});

		it('should show +N indicator for extra tags', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent({ tags: ['tag1', 'tag2', 'tag3', 'tag4', 'tag5'] }) }
			});

			expect(screen.getByText('+3')).toBeInTheDocument();
		});
	});

	describe('actions menu', () => {
		it('should show menu button when showActions is true', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent(), showActions: true }
			});

			const buttons = screen.getAllByRole('button');
			expect(buttons.length).toBeGreaterThan(0);
		});

		it('should not show menu button when showActions is false', () => {
			render(SubAgentCard, {
				props: { subagent: createMockSubAgent(), showActions: false }
			});

			expect(screen.queryByRole('button')).not.toBeInTheDocument();
		});

		it('should call onEdit when Edit is clicked', async () => {
			const onEdit = vi.fn();
			const subagent = createMockSubAgent();
			render(SubAgentCard, { props: { subagent, onEdit } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Edit'));

			expect(onEdit).toHaveBeenCalledWith(subagent);
		});

		it('should call onDelete when Delete is clicked', async () => {
			const onDelete = vi.fn();
			const subagent = createMockSubAgent();
			render(SubAgentCard, { props: { subagent, onDelete } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Delete'));

			expect(onDelete).toHaveBeenCalledWith(subagent);
		});

		it('should close menu after action', async () => {
			const onEdit = vi.fn();
			render(SubAgentCard, { props: { subagent: createMockSubAgent(), onEdit } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Edit'));

			expect(screen.queryByText('Edit')).not.toBeInTheDocument();
		});
	});
});
