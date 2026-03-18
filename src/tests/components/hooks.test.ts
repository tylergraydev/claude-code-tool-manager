import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte/pure';

vi.mock('$lib/stores', () => ({
	hookLibrary: {
		hooks: [],
		filteredHooks: [],
		isLoading: false,
		searchQuery: '',
		eventFilter: '',
		viewMode: 'all',
		error: null,
		load: vi.fn(),
		create: vi.fn(),
		delete: vi.fn(),
		toggleFavorite: vi.fn(),
		getProjectHooks: vi.fn().mockResolvedValue([]),
		getHookById: vi.fn(),
		assignToProject: vi.fn(),
		removeFromProject: vi.fn(),
		toggleProjectHook: vi.fn(),
		globalHooks: [],
		loadGlobalHooks: vi.fn(),
		addGlobalHook: vi.fn(),
		removeGlobalHook: vi.fn(),
		toggleGlobalHook: vi.fn(),
		groupedByEvent: {},
		hooksByEventType: [],
		projectsWithHooks: [],
		unassignedHooks: [],
		setEventFilter: vi.fn(),
		setViewMode: vi.fn(),
		exportToJson: vi.fn().mockResolvedValue('{}'),
		exportToClipboard: vi.fn(),
		createSoundNotificationHooks: vi.fn().mockResolvedValue([])
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	},
	soundLibrary: {
		systemSounds: [],
		load: vi.fn(),
		getSoundByPath: vi.fn(),
		deployNotificationScript: vi.fn()
	}
}));

vi.mock('$lib/types', async (importOriginal) => {
	const actual = await importOriginal() as any;
	return {
		...actual,
		HOOK_EVENT_TYPES: actual.HOOK_EVENT_TYPES ?? [
			{ value: 'PreToolUse', label: 'Pre Tool Use', description: 'Before tool runs', matcherHint: 'Tool name' },
			{ value: 'PostToolUse', label: 'Post Tool Use', description: 'After tool runs', matcherHint: 'Tool name' },
			{ value: 'Notification', label: 'Notification', description: 'Notification events' },
			{ value: 'Stop', label: 'Stop', description: 'When Claude stops' },
			{ value: 'SubagentStop', label: 'Subagent Stop', description: 'When subagent stops' }
		],
		SOUND_HOOK_PRESETS: actual.SOUND_HOOK_PRESETS ?? [
			{ id: 'task-complete', name: 'Task Complete', description: 'Sound on task finish', events: ['Stop'] },
			{ id: 'permission-required', name: 'Permission Required', description: 'Sound on permission', events: ['Notification'] },
			{ id: 'full-suite', name: 'Full Suite', description: 'All events', events: ['Stop', 'SubagentStop', 'Notification'] }
		],
		getDefaultSound: actual.getDefaultSound ?? (() => '/System/Library/Sounds/Glass.aiff')
	};
});

vi.mock('@tauri-apps/plugin-dialog', () => ({
	save: vi.fn()
}));

vi.mock('@tauri-apps/plugin-fs', () => ({
	writeTextFile: vi.fn()
}));

vi.mock('$lib/components/sounds', () => ({
	SoundPicker: {}
}));

describe('HookCard Component', () => {
	let HookCard: any;

	const mockHook = {
		id: 1,
		name: 'Test Hook',
		hookType: 'command' as const,
		eventType: 'PreToolUse',
		matcher: 'Bash',
		command: 'echo test',
		isEnabled: true,
		isFavorite: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/hooks/HookCard.svelte');
		HookCard = mod.default;
	});

	it('should render hook name', () => {
		render(HookCard, { props: { hook: mockHook } });
		expect(screen.getByText('Test Hook')).toBeInTheDocument();
	});

	it('should show event type badge', () => {
		render(HookCard, { props: { hook: mockHook } });
		expect(screen.getAllByText('PreToolUse').length).toBeGreaterThan(0);
	});

	it('should show matcher', () => {
		render(HookCard, { props: { hook: mockHook } });
		expect(screen.getAllByText('Bash').length).toBeGreaterThan(0);
	});

	it('should hide actions when showActions is false', () => {
		render(HookCard, {
			props: { hook: mockHook, showActions: false }
		});
		// No menu button should be rendered
		const buttons = document.querySelectorAll('button');
		// Only the window listener, no action buttons
		expect(screen.queryByLabelText(/Actions/)).not.toBeInTheDocument();
	});

	it('should show actions menu button by default', () => {
		render(HookCard, { props: { hook: mockHook } });
		// The MoreVertical icon button exists
		const buttons = document.querySelectorAll('button');
		expect(buttons.length).toBeGreaterThan(0);
	});

	it('should show Command badge for command hook type', () => {
		render(HookCard, { props: { hook: mockHook } });
		expect(screen.getByText('Command')).toBeInTheDocument();
	});

	it('should show Prompt badge for prompt hook type', () => {
		const promptHook = { ...mockHook, hookType: 'prompt' as const, prompt: 'test prompt' };
		render(HookCard, { props: { hook: promptHook } });
		expect(screen.getByText('Prompt')).toBeInTheDocument();
	});

	it('should show Template badge when isTemplate is true', () => {
		const templateHook = { ...mockHook, isTemplate: true };
		render(HookCard, { props: { hook: templateHook } });
		expect(screen.getByText('Template')).toBeInTheDocument();
	});

	it('should not show Template badge when isTemplate is false', () => {
		render(HookCard, { props: { hook: mockHook } });
		expect(screen.queryByText('Template')).not.toBeInTheDocument();
	});

	it('should show description when provided', () => {
		const hookWithDesc = { ...mockHook, description: 'A test description' };
		render(HookCard, { props: { hook: hookWithDesc } });
		expect(screen.getByText('A test description')).toBeInTheDocument();
	});

	it('should not show description when not provided', () => {
		render(HookCard, { props: { hook: mockHook } });
		// No description paragraph should be rendered
		expect(screen.queryByText(/description/i)).not.toBeInTheDocument();
	});

	it('should not show matcher when not provided', () => {
		const hookNoMatcher = { ...mockHook, matcher: '' };
		render(HookCard, { props: { hook: hookNoMatcher } });
		// Bash matcher should not appear
		expect(screen.queryByText('Bash')).not.toBeInTheDocument();
	});

	it('should show timeout badge when timeout is set', () => {
		const hookWithTimeout = { ...mockHook, timeout: 30 };
		render(HookCard, { props: { hook: hookWithTimeout } });
		expect(screen.getByText('30s timeout')).toBeInTheDocument();
	});

	it('should not show timeout badge when timeout is not set', () => {
		render(HookCard, { props: { hook: mockHook } });
		expect(screen.queryByText(/timeout/)).not.toBeInTheDocument();
	});

	it('should show tags', () => {
		const hookWithTags = { ...mockHook, tags: ['formatting', 'security'] };
		render(HookCard, { props: { hook: hookWithTags } });
		expect(screen.getByText('formatting')).toBeInTheDocument();
		expect(screen.getByText('security')).toBeInTheDocument();
	});

	it('should show only first 2 tags and overflow count', () => {
		const hookWithManyTags = { ...mockHook, tags: ['tag1', 'tag2', 'tag3', 'tag4'] };
		render(HookCard, { props: { hook: hookWithManyTags } });
		expect(screen.getByText('tag1')).toBeInTheDocument();
		expect(screen.getByText('tag2')).toBeInTheDocument();
		expect(screen.queryByText('tag3')).not.toBeInTheDocument();
		expect(screen.getByText('+2')).toBeInTheDocument();
	});

	it('should show different event type colors', () => {
		const postHook = { ...mockHook, eventType: 'PostToolUse' };
		render(HookCard, { props: { hook: postHook } });
		expect(screen.getAllByText('PostToolUse').length).toBeGreaterThan(0);
	});

	it('should show Notification event type', () => {
		const notifHook = { ...mockHook, eventType: 'Notification' };
		render(HookCard, { props: { hook: notifHook } });
		expect(screen.getAllByText('Notification').length).toBeGreaterThan(0);
	});

	it('should show Stop event type', () => {
		const stopHook = { ...mockHook, eventType: 'Stop' };
		render(HookCard, { props: { hook: stopHook } });
		expect(screen.getAllByText('Stop').length).toBeGreaterThan(0);
	});

	it('should show SubagentStop event type', () => {
		const subHook = { ...mockHook, eventType: 'SubagentStop' };
		render(HookCard, { props: { hook: subHook } });
		expect(screen.getAllByText('SubagentStop').length).toBeGreaterThan(0);
	});
});

describe('HookForm Component', () => {
	let HookForm: any;

	const defaultProps = {
		onSubmit: vi.fn(),
		onCancel: vi.fn()
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/hooks/HookForm.svelte');
		HookForm = mod.default;
	});

	it('should render description field', () => {
		render(HookForm, { props: defaultProps });
		expect(screen.getByLabelText(/Description/)).toBeInTheDocument();
	});

	it('should render event type selector', () => {
		render(HookForm, { props: defaultProps });
		expect(screen.getByText('Event Type')).toBeInTheDocument();
	});

	it('should render matcher field', () => {
		render(HookForm, { props: defaultProps });
		expect(screen.getByLabelText(/Matcher Pattern/)).toBeInTheDocument();
	});

	it('should render hook type toggle with Command and Prompt options', () => {
		render(HookForm, { props: defaultProps });
		expect(screen.getAllByText('Command').length).toBeGreaterThan(0);
		expect(screen.getAllByText('Prompt').length).toBeGreaterThan(0);
	});

	it('should show command textarea by default', () => {
		render(HookForm, { props: defaultProps });
		expect(screen.getByLabelText(/^Command/)).toBeInTheDocument();
	});

	it('should show timeout field for command type', () => {
		render(HookForm, { props: defaultProps });
		expect(screen.getByLabelText(/Timeout/)).toBeInTheDocument();
	});

	it('should show prompt textarea when prompt type selected', async () => {
		render(HookForm, { props: defaultProps });
		const promptBtn = screen.getByText('Prompt');
		await fireEvent.click(promptBtn);
		expect(screen.getByLabelText(/Prompt Text/)).toBeInTheDocument();
	});

	it('should hide command textarea when prompt type selected', async () => {
		render(HookForm, { props: defaultProps });
		const promptBtn = screen.getByText('Prompt');
		await fireEvent.click(promptBtn);
		expect(screen.queryByLabelText(/^Command /)).not.toBeInTheDocument();
	});

	it('should hide timeout field when prompt type selected', async () => {
		render(HookForm, { props: defaultProps });
		const promptBtn = screen.getByText('Prompt');
		await fireEvent.click(promptBtn);
		expect(screen.queryByLabelText(/Timeout/)).not.toBeInTheDocument();
	});

	it('should show tags field', () => {
		render(HookForm, { props: defaultProps });
		expect(screen.getByLabelText(/Tags/)).toBeInTheDocument();
	});

	it('should show Create Hook button when no initial values', () => {
		render(HookForm, { props: defaultProps });
		expect(screen.getByText('Create Hook')).toBeInTheDocument();
	});

	it('should show Update Hook button when editing', () => {
		render(HookForm, {
			props: { ...defaultProps, initialValues: { name: 'existing-hook' } }
		});
		expect(screen.getByText('Update Hook')).toBeInTheDocument();
	});

	it('should show Cancel button', () => {
		render(HookForm, { props: defaultProps });
		expect(screen.getByText('Cancel')).toBeInTheDocument();
	});

	it('should call onCancel when cancel clicked', async () => {
		const onCancel = vi.fn();
		render(HookForm, { props: { ...defaultProps, onCancel } });
		await fireEvent.click(screen.getByText('Cancel'));
		expect(onCancel).toHaveBeenCalled();
	});

	it('should validate command is required for command hooks', async () => {
		render(HookForm, { props: defaultProps });
		const form = document.querySelector('form')!;
		await fireEvent.submit(form);
		expect(screen.getByText('Command is required for command hooks')).toBeInTheDocument();
	});

	it('should validate prompt is required for prompt hooks', async () => {
		render(HookForm, { props: defaultProps });
		const promptBtn = screen.getByText('Prompt');
		await fireEvent.click(promptBtn);
		const form = document.querySelector('form')!;
		await fireEvent.submit(form);
		expect(screen.getByText('Prompt is required for prompt hooks')).toBeInTheDocument();
	});

	it('should validate timeout is a positive number', async () => {
		render(HookForm, { props: defaultProps });
		const commandInput = screen.getByLabelText(/^Command/);
		await fireEvent.input(commandInput, { target: { value: 'echo test' } });
		const timeoutInput = screen.getByLabelText(/Timeout/);
		await fireEvent.input(timeoutInput, { target: { value: '-5' } });
		const form = document.querySelector('form')!;
		await fireEvent.submit(form);
		expect(screen.getByText('Timeout must be a positive number')).toBeInTheDocument();
	});

	it('should submit with correct command data', async () => {
		const onSubmit = vi.fn();
		render(HookForm, { props: { ...defaultProps, onSubmit } });
		const commandInput = screen.getByLabelText(/^Command/);
		await fireEvent.input(commandInput, { target: { value: 'echo hello' } });
		const form = document.querySelector('form')!;
		await fireEvent.submit(form);
		expect(onSubmit).toHaveBeenCalledWith(
			expect.objectContaining({
				hookType: 'command',
				command: 'echo hello'
			})
		);
	});

	it('should submit with correct prompt data', async () => {
		const onSubmit = vi.fn();
		render(HookForm, { props: { ...defaultProps, onSubmit } });
		const promptBtn = screen.getByText('Prompt');
		await fireEvent.click(promptBtn);
		const promptInput = screen.getByLabelText(/Prompt Text/);
		await fireEvent.input(promptInput, { target: { value: 'Test prompt' } });
		const form = document.querySelector('form')!;
		await fireEvent.submit(form);
		expect(onSubmit).toHaveBeenCalledWith(
			expect.objectContaining({
				hookType: 'prompt',
				prompt: 'Test prompt'
			})
		);
	});

	it('should populate initial values', () => {
		render(HookForm, {
			props: {
				...defaultProps,
				initialValues: {
					name: 'test-hook',
					description: 'A hook',
					eventType: 'PreToolUse',
					matcher: 'Bash',
					hookType: 'command',
					command: 'echo test',
					timeout: 30,
					tags: ['tag1', 'tag2']
				}
			}
		});
		expect(screen.getByDisplayValue('A hook')).toBeInTheDocument();
		expect(screen.getByDisplayValue('Bash')).toBeInTheDocument();
		expect(screen.getByDisplayValue('echo test')).toBeInTheDocument();
		expect(screen.getByDisplayValue('30')).toBeInTheDocument();
		expect(screen.getByDisplayValue('tag1, tag2')).toBeInTheDocument();
	});

	it('should show import section', () => {
		render(HookForm, { props: defaultProps });
		expect(screen.getByText('Import from JSON or Template')).toBeInTheDocument();
	});

	it('should show Paste and File buttons', () => {
		render(HookForm, { props: defaultProps });
		expect(screen.getByText('Paste')).toBeInTheDocument();
		expect(screen.getByText('File')).toBeInTheDocument();
	});

	it('should show template dropdown when templates provided', () => {
		const templates = [
			{ id: 1, name: 'Template 1', hookType: 'command', eventType: 'PreToolUse', isEnabled: true, isTemplate: true, command: 'echo', createdAt: '', updatedAt: '' }
		];
		render(HookForm, { props: { ...defaultProps, templates } });
		expect(screen.getByText('Templates...')).toBeInTheDocument();
	});

	it('should parse tags from comma-separated input', async () => {
		const onSubmit = vi.fn();
		render(HookForm, { props: { ...defaultProps, onSubmit } });
		const commandInput = screen.getByLabelText(/^Command/);
		await fireEvent.input(commandInput, { target: { value: 'echo test' } });
		const tagsInput = screen.getByLabelText(/Tags/);
		await fireEvent.input(tagsInput, { target: { value: 'tag1, tag2, tag3' } });
		const form = document.querySelector('form')!;
		await fireEvent.submit(form);
		expect(onSubmit).toHaveBeenCalledWith(
			expect.objectContaining({
				tags: ['tag1', 'tag2', 'tag3']
			})
		);
	});

	it('should include matcher in submission when provided', async () => {
		const onSubmit = vi.fn();
		render(HookForm, { props: { ...defaultProps, onSubmit } });
		const matcherInput = screen.getByLabelText(/Matcher Pattern/);
		await fireEvent.input(matcherInput, { target: { value: 'Bash|Write' } });
		const commandInput = screen.getByLabelText(/^Command/);
		await fireEvent.input(commandInput, { target: { value: 'echo test' } });
		const form = document.querySelector('form')!;
		await fireEvent.submit(form);
		expect(onSubmit).toHaveBeenCalledWith(
			expect.objectContaining({
				matcher: 'Bash|Write'
			})
		);
	});
});

describe('HookLibrary Component', () => {
	let HookLibrary: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/hooks/HookLibrary.svelte');
		HookLibrary = mod.default;
	});

	it('should render hook library', () => {
		render(HookLibrary, { props: {} });
		expect(document.body).toBeTruthy();
	});

	it('should show search bar', () => {
		render(HookLibrary, { props: {} });
		expect(screen.getByPlaceholderText('Search hooks...')).toBeInTheDocument();
	});

	it('should show All Events filter dropdown', () => {
		render(HookLibrary, { props: {} });
		expect(screen.getByText('All Events')).toBeInTheDocument();
	});

	it('should show view mode toggle with All and By Scope', () => {
		render(HookLibrary, { props: {} });
		expect(screen.getByText('All')).toBeInTheDocument();
		expect(screen.getByText('By Scope')).toBeInTheDocument();
	});

	it('should show hook count', () => {
		render(HookLibrary, { props: {} });
		expect(screen.getByText('0 hooks')).toBeInTheDocument();
	});

	it('should show empty state for all view when no hooks', () => {
		render(HookLibrary, { props: {} });
		expect(screen.getByText('No hooks in library')).toBeInTheDocument();
	});

	it('should show add hook description in empty state', () => {
		render(HookLibrary, { props: {} });
		expect(screen.getByText('Add your first hook to automate Claude Code actions')).toBeInTheDocument();
	});
});

describe('HookExportModal Component', () => {
	let HookExportModal: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/hooks/HookExportModal.svelte');
		HookExportModal = mod.default;
	});

	it('should render export modal', () => {
		render(HookExportModal, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Export Hooks')).toBeInTheDocument();
	});

	it('should show "Select hooks to export as JSON" description', () => {
		render(HookExportModal, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Select hooks to export as JSON')).toBeInTheDocument();
	});

	it('should show Select all and Clear buttons', () => {
		render(HookExportModal, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Select all')).toBeInTheDocument();
		expect(screen.getByText('Clear')).toBeInTheDocument();
	});

	it('should show selection count', () => {
		render(HookExportModal, { props: { onClose: vi.fn() } });
		expect(screen.getByText('0 of 0 selected')).toBeInTheDocument();
	});

	it('should show Copy to Clipboard button', () => {
		render(HookExportModal, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Copy to Clipboard')).toBeInTheDocument();
	});

	it('should show Export to File button', () => {
		render(HookExportModal, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Export to File')).toBeInTheDocument();
	});

	it('should show Cancel button', () => {
		render(HookExportModal, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Cancel')).toBeInTheDocument();
	});

	it('should show empty state when no hooks available', () => {
		render(HookExportModal, { props: { onClose: vi.fn() } });
		expect(screen.getByText('No hooks to export')).toBeInTheDocument();
	});

	it('should show preview placeholder when no hooks selected', () => {
		render(HookExportModal, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Select hooks to preview export')).toBeInTheDocument();
	});

	it('should show Preview section header', () => {
		render(HookExportModal, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Preview')).toBeInTheDocument();
	});
});

describe('SoundHookWizard Component', () => {
	let SoundHookWizard: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/hooks/SoundHookWizard.svelte');
		SoundHookWizard = mod.default;
	});

	it('should render wizard title', () => {
		render(SoundHookWizard, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Sound Notifications Setup')).toBeInTheDocument();
	});

	it('should show Step 1 of 3', () => {
		render(SoundHookWizard, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Step 1 of 3')).toBeInTheDocument();
	});

	it('should show Choose Events heading', () => {
		render(SoundHookWizard, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Choose Events')).toBeInTheDocument();
	});

	it('should show Quick Presets section', () => {
		render(SoundHookWizard, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Quick Presets')).toBeInTheDocument();
	});

	it('should show individual events section', () => {
		render(SoundHookWizard, { props: { onClose: vi.fn() } });
		expect(screen.getByText('Or Select Individual Events')).toBeInTheDocument();
	});

	it('should show event options', () => {
		render(SoundHookWizard, { props: { onClose: vi.fn() } });
		expect(screen.getAllByText('Task Complete').length).toBeGreaterThan(0);
		expect(screen.getAllByText('Notification').length).toBeGreaterThan(0);
	});

	it('should show Back button (disabled on step 1)', () => {
		render(SoundHookWizard, { props: { onClose: vi.fn() } });
		const backBtn = screen.getByText('Back');
		expect(backBtn.closest('button')?.disabled).toBe(true);
	});

	it('should show Next button (disabled when no events selected)', () => {
		render(SoundHookWizard, { props: { onClose: vi.fn() } });
		const nextBtn = screen.getByText('Next');
		expect(nextBtn.closest('button')?.disabled).toBe(true);
	});

	it('should enable Next button when an event is selected', async () => {
		render(SoundHookWizard, { props: { onClose: vi.fn() } });
		// Click a checkbox event - multiple elements exist for 'Task Complete', get the one in individual events
		const allTaskComplete = screen.getAllByText('Task Complete');
		// The last one should be the individual event button
		const lastTaskComplete = allTaskComplete[allTaskComplete.length - 1];
		await fireEvent.click(lastTaskComplete.closest('button')!);
		const nextBtn = screen.getByText('Next');
		expect(nextBtn.closest('button')?.disabled).toBe(false);
	});
});

describe('Hooks index.ts exports', () => {
	let hookExports: any;

	beforeAll(async () => {
		hookExports = await import('$lib/components/hooks');
	});

	it('should export all components', () => {
		expect(hookExports.HookCard).toBeDefined();
		expect(hookExports.HookExportModal).toBeDefined();
		expect(hookExports.HookForm).toBeDefined();
		expect(hookExports.HookLibrary).toBeDefined();
		expect(hookExports.SoundHookWizard).toBeDefined();
	});
});
