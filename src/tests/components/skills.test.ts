import { describe, it, expect, vi, beforeAll, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	skillLibrary: {
		skills: [],
		filteredSkills: [],
		isLoading: false,
		searchQuery: '',
		load: vi.fn(),
		create: vi.fn(),
		delete: vi.fn(),
		getSkillById: vi.fn(),
		getProjectSkills: vi.fn().mockResolvedValue([]),
		assignToProject: vi.fn(),
		removeFromProject: vi.fn(),
		toggleProjectSkill: vi.fn(),
		globalSkills: [],
		loadGlobalSkills: vi.fn(),
		addGlobalSkill: vi.fn(),
		removeGlobalSkill: vi.fn(),
		toggleGlobalSkill: vi.fn(),
		updateSkill: vi.fn(),
		getSkillFiles: vi.fn().mockResolvedValue([]),
		createSkillFile: vi.fn(),
		updateSkillFile: vi.fn(),
		deleteSkillFile: vi.fn()
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

vi.mock('$lib/utils/markdownParser', () => ({
	parseSkillMarkdown: vi.fn().mockReturnValue({ success: false })
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn().mockResolvedValue(null)
}));

describe('SkillCard Component', () => {
	let SkillCard: any;

	const mockSkill = {
		id: 1,
		name: 'My Skill',
		description: 'A test skill',
		content: 'Do something',
		allowedTools: ['Read', 'Write'],
		tags: ['test', 'dev', 'ops'],
		source: 'user' as const,
		isFavorite: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/skills/SkillCard.svelte');
		SkillCard = mod.default;
	});

	it('should render skill name', () => {
		render(SkillCard, { props: { skill: mockSkill } });
		expect(screen.getByText('My Skill')).toBeInTheDocument();
	});

	it('should render description when present', () => {
		render(SkillCard, { props: { skill: mockSkill } });
		expect(screen.getByText('A test skill')).toBeInTheDocument();
	});

	it('should not render description when absent', () => {
		render(SkillCard, { props: { skill: { ...mockSkill, description: '' } } });
		expect(screen.queryByText('A test skill')).not.toBeInTheDocument();
	});

	it('should show auto-detected badge when source is auto-detected', () => {
		render(SkillCard, { props: { skill: { ...mockSkill, source: 'auto-detected' } } });
		expect(screen.getByText('Auto')).toBeInTheDocument();
	});

	it('should not show auto-detected badge for user source', () => {
		render(SkillCard, { props: { skill: mockSkill } });
		expect(screen.queryByText('Auto')).not.toBeInTheDocument();
	});

	it('should display tool count badge', () => {
		render(SkillCard, { props: { skill: mockSkill } });
		expect(screen.getByText('2 tools')).toBeInTheDocument();
	});

	it('should display singular tool text for one tool', () => {
		render(SkillCard, { props: { skill: { ...mockSkill, allowedTools: ['Read'] } } });
		expect(screen.getByText('1 tool')).toBeInTheDocument();
	});

	it('should not show tool badge when no tools', () => {
		render(SkillCard, { props: { skill: { ...mockSkill, allowedTools: [] } } });
		expect(screen.queryByText(/tool/)).not.toBeInTheDocument();
	});

	it('should show Manual only badge when disableModelInvocation is true', () => {
		render(SkillCard, { props: { skill: { ...mockSkill, disableModelInvocation: true } } });
		expect(screen.getByText('Manual only')).toBeInTheDocument();
	});

	it('should show first two tags and overflow count', () => {
		render(SkillCard, { props: { skill: mockSkill } });
		expect(screen.getByText('test')).toBeInTheDocument();
		expect(screen.getByText('dev')).toBeInTheDocument();
		expect(screen.getByText('+1')).toBeInTheDocument();
	});

	it('should not show overflow when 2 or fewer tags', () => {
		render(SkillCard, { props: { skill: { ...mockSkill, tags: ['a', 'b'] } } });
		expect(screen.queryByText(/\+/)).not.toBeInTheDocument();
	});

	it('should hide actions when showActions is false', () => {
		const { container } = render(SkillCard, {
			props: { skill: mockSkill, showActions: false }
		});
		// No favorite or menu buttons when actions hidden
		const buttons = container.querySelectorAll('button');
		expect(buttons.length).toBe(0);
	});

	it('should show favorite button when onFavoriteToggle is provided', () => {
		const onFavoriteToggle = vi.fn();
		render(SkillCard, {
			props: { skill: mockSkill, onFavoriteToggle }
		});
		expect(screen.getByTitle('Add to favorites')).toBeInTheDocument();
	});

	it('should show Remove from favorites title when isFavorite', () => {
		render(SkillCard, {
			props: { skill: { ...mockSkill, isFavorite: true }, onFavoriteToggle: vi.fn() }
		});
		expect(screen.getByTitle('Remove from favorites')).toBeInTheDocument();
	});

	it('should call onFavoriteToggle when favorite button clicked', async () => {
		const onFavoriteToggle = vi.fn();
		render(SkillCard, {
			props: { skill: mockSkill, onFavoriteToggle }
		});
		await fireEvent.click(screen.getByTitle('Add to favorites'));
		expect(onFavoriteToggle).toHaveBeenCalledWith(mockSkill, true);
	});
});

describe('SkillForm Component', () => {
	let SkillForm: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/skills/SkillForm.svelte');
		SkillForm = mod.default;
	});

	it('should render form with all fields', () => {
		render(SkillForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByLabelText(/Name/)).toBeInTheDocument();
		expect(screen.getByLabelText(/Description/)).toBeInTheDocument();
		expect(screen.getByLabelText(/Allowed Tools/)).toBeInTheDocument();
		expect(screen.getByLabelText(/Model Override/)).toBeInTheDocument();
		expect(screen.getByLabelText(/Skill Instructions/)).toBeInTheDocument();
		expect(screen.getByLabelText(/Tags/)).toBeInTheDocument();
	});

	it('should show Create Skill button for new skill', () => {
		render(SkillForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Create Skill')).toBeInTheDocument();
	});

	it('should show Update Skill button when editing', () => {
		render(SkillForm, {
			props: { initialValues: { name: 'test' }, onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByText('Update Skill')).toBeInTheDocument();
	});

	it('should show Import from Markdown section', () => {
		render(SkillForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Import from Markdown')).toBeInTheDocument();
	});

	it('should show File and Paste buttons', () => {
		render(SkillForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('File')).toBeInTheDocument();
		expect(screen.getByText('Paste')).toBeInTheDocument();
	});

	it('should call onCancel when Cancel button clicked', async () => {
		const onCancel = vi.fn();
		render(SkillForm, { props: { onSubmit: vi.fn(), onCancel } });
		await fireEvent.click(screen.getByText('Cancel'));
		expect(onCancel).toHaveBeenCalledOnce();
	});

	it('should show Disable Model Invocation checkbox', () => {
		render(SkillForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByLabelText(/Disable Model Invocation/)).toBeInTheDocument();
	});
});

describe('SkillLibrary Component', () => {
	let SkillLibrary: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/skills/SkillLibrary.svelte');
		SkillLibrary = mod.default;
	});

	it('should render empty state when no skills', () => {
		render(SkillLibrary);
		expect(screen.getByText('No skills in library')).toBeInTheDocument();
	});

	it('should render search bar placeholder', () => {
		render(SkillLibrary);
		expect(screen.getByPlaceholderText('Search skills...')).toBeInTheDocument();
	});

	it('should show skill count', () => {
		render(SkillLibrary);
		expect(screen.getByText('0 skills')).toBeInTheDocument();
	});
});

describe('SkillFilesEditor Component', () => {
	let SkillFilesEditor: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/skills/SkillFilesEditor.svelte');
		SkillFilesEditor = mod.default;
	});

	it('should render the Skill Files heading', () => {
		render(SkillFilesEditor, { props: { skillId: 1, skillName: 'test-skill' } });
		expect(screen.getByText('Skill Files')).toBeInTheDocument();
	});

	it('should show the skill path info', () => {
		render(SkillFilesEditor, { props: { skillId: 1, skillName: 'test-skill' } });
		expect(screen.getByText(/\.claude\/skills\/test-skill/)).toBeInTheDocument();
	});

	it('should show Add File button initially', () => {
		render(SkillFilesEditor, { props: { skillId: 1, skillName: 'test-skill' } });
		expect(screen.getByText('Add File')).toBeInTheDocument();
	});
});

describe('Skills index.ts exports', () => {
	let skillExports: any;

	beforeAll(async () => {
		skillExports = await import('$lib/components/skills');
	});

	it('should export all components', () => {
		expect(skillExports.SkillCard).toBeDefined();
		expect(skillExports.SkillLibrary).toBeDefined();
		expect(skillExports.SkillForm).toBeDefined();
		expect(skillExports.SkillFilesEditor).toBeDefined();
	});
});
