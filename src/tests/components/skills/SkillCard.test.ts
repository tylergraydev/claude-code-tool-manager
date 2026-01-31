import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import SkillCard from '$lib/components/skills/SkillCard.svelte';
import type { Skill } from '$lib/types';

describe('SkillCard', () => {
	const createMockSkill = (overrides: Partial<Skill> = {}): Skill => ({
		id: 1,
		name: 'Test Skill',
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
		it('should render skill name', () => {
			render(SkillCard, { props: { skill: createMockSkill({ name: 'My Custom Skill' }) } });

			expect(screen.getByText('My Custom Skill')).toBeInTheDocument();
		});

		it('should render skill description when provided', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ description: 'A helpful skill' }) }
			});

			expect(screen.getByText('A helpful skill')).toBeInTheDocument();
		});

		it('should not render description when not provided', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ description: undefined }) }
			});

			expect(screen.queryByText(/helpful skill/i)).not.toBeInTheDocument();
		});

		it('should have Sparkles icon', () => {
			const { container } = render(SkillCard, { props: { skill: createMockSkill() } });

			const iconContainer = container.querySelector('.bg-purple-100');
			expect(iconContainer).toBeInTheDocument();
		});
	});

	describe('source badges', () => {
		it('should show Auto badge for auto-detected skills', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ source: 'auto-detected' }) }
			});

			expect(screen.getByText('Auto')).toBeInTheDocument();
		});

		it('should not show Auto badge for user skills', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ source: 'user' }) }
			});

			expect(screen.queryByText('Auto')).not.toBeInTheDocument();
		});
	});

	describe('allowed tools badge', () => {
		it('should show tool count when tools are allowed', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ allowedTools: ['tool1', 'tool2', 'tool3'] }) }
			});

			expect(screen.getByText('3 tools')).toBeInTheDocument();
		});

		it('should use singular form for 1 tool', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ allowedTools: ['tool1'] }) }
			});

			expect(screen.getByText('1 tool')).toBeInTheDocument();
		});

		it('should not show tools badge when no tools allowed', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ allowedTools: [] }) }
			});

			expect(screen.queryByText(/tool/i)).not.toBeInTheDocument();
		});

		it('should not show tools badge when allowedTools is undefined', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ allowedTools: undefined }) }
			});

			expect(screen.queryByText(/tool/i)).not.toBeInTheDocument();
		});
	});

	describe('manual only badge', () => {
		it('should show Manual only badge when disableModelInvocation is true', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ disableModelInvocation: true }) }
			});

			expect(screen.getByText('Manual only')).toBeInTheDocument();
		});

		it('should not show Manual only badge when disableModelInvocation is false', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ disableModelInvocation: false }) }
			});

			expect(screen.queryByText('Manual only')).not.toBeInTheDocument();
		});
	});

	describe('tags', () => {
		it('should show tags when provided', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ tags: ['tag1', 'tag2'] }) }
			});

			expect(screen.getByText('tag1')).toBeInTheDocument();
			expect(screen.getByText('tag2')).toBeInTheDocument();
		});

		it('should show only first 2 tags', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ tags: ['tag1', 'tag2', 'tag3', 'tag4'] }) }
			});

			expect(screen.getByText('tag1')).toBeInTheDocument();
			expect(screen.getByText('tag2')).toBeInTheDocument();
			expect(screen.queryByText('tag3')).not.toBeInTheDocument();
			expect(screen.queryByText('tag4')).not.toBeInTheDocument();
		});

		it('should show +N indicator for extra tags', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ tags: ['tag1', 'tag2', 'tag3', 'tag4', 'tag5'] }) }
			});

			expect(screen.getByText('+3')).toBeInTheDocument();
		});

		it('should not show tags section when empty', () => {
			render(SkillCard, {
				props: { skill: createMockSkill({ tags: [] }) }
			});

			expect(screen.queryByText(/^\+\d+$/)).not.toBeInTheDocument();
		});
	});

	describe('actions menu', () => {
		it('should show menu button when showActions is true', () => {
			render(SkillCard, {
				props: { skill: createMockSkill(), showActions: true }
			});

			const buttons = screen.getAllByRole('button');
			expect(buttons.length).toBeGreaterThan(0);
		});

		it('should not show menu button when showActions is false', () => {
			render(SkillCard, {
				props: { skill: createMockSkill(), showActions: false }
			});

			expect(screen.queryByRole('button')).not.toBeInTheDocument();
		});

		it('should toggle menu on button click', async () => {
			const onEdit = vi.fn();
			render(SkillCard, {
				props: { skill: createMockSkill(), onEdit }
			});

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);

			expect(screen.getByText('Edit')).toBeInTheDocument();
		});

		it('should call onEdit when Edit is clicked', async () => {
			const onEdit = vi.fn();
			const skill = createMockSkill();
			render(SkillCard, { props: { skill, onEdit } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Edit'));

			expect(onEdit).toHaveBeenCalledWith(skill);
		});

		it('should call onDelete when Delete is clicked', async () => {
			const onDelete = vi.fn();
			const skill = createMockSkill();
			render(SkillCard, { props: { skill, onDelete } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Delete'));

			expect(onDelete).toHaveBeenCalledWith(skill);
		});

		it('should close menu after action', async () => {
			const onEdit = vi.fn();
			render(SkillCard, { props: { skill: createMockSkill(), onEdit } });

			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);
			await fireEvent.click(screen.getByText('Edit'));

			expect(screen.queryByText('Edit')).not.toBeInTheDocument();
		});
	});
});
