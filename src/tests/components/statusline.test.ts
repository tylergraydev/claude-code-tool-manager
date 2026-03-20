import { describe, it, expect, vi, beforeAll, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	statuslineLibrary: {
		statuslines: [],
		filteredStatusLines: [],
		gallery: [],
		isLoading: false,
		isGalleryLoading: false,
		searchQuery: '',
		load: vi.fn(),
		create: vi.fn(),
		delete: vi.fn(),
		activate: vi.fn(),
		deactivate: vi.fn(),
		galleryItems: [],
		loadGallery: vi.fn(),
		setSearch: vi.fn(),
		generatePreview: vi.fn().mockResolvedValue('# preview script')
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

vi.mock('svelte-dnd-action', () => ({
	dndzone: () => ({ destroy: () => {} })
}));

// ──────────────────────────────────────────────────────────
// StatusLineCard
// ──────────────────────────────────────────────────────────
describe('StatusLineCard Component', () => {
	let StatusLineCard: any;

	const mockStatusLine = {
		id: 1,
		name: 'Test Status',
		statuslineType: 'custom' as const,
		segments: [],
		rawContent: '',
		isActive: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01',
		description: '',
		author: '',
		homepageUrl: ''
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/statusline/StatusLineCard.svelte');
		StatusLineCard = mod.default;
	});

	it('should render status line name', () => {
		render(StatusLineCard, { props: { statusline: mockStatusLine } });
		expect(screen.getByText('Test Status')).toBeInTheDocument();
	});

	it('should show type badge Custom', () => {
		render(StatusLineCard, { props: { statusline: mockStatusLine } });
		expect(screen.getAllByText('Custom').length).toBeGreaterThan(0);
	});

	it('should show Premade badge for premade type', () => {
		const premade = { ...mockStatusLine, statuslineType: 'premade' };
		render(StatusLineCard, { props: { statusline: premade } });
		expect(screen.getAllByText('Premade').length).toBeGreaterThan(0);
	});

	it('should show Raw badge for raw type', () => {
		const raw = { ...mockStatusLine, statuslineType: 'raw' };
		render(StatusLineCard, { props: { statusline: raw } });
		expect(screen.getAllByText('Raw').length).toBeGreaterThan(0);
	});

	it('should show Active badge when isActive', () => {
		const active = { ...mockStatusLine, isActive: true };
		render(StatusLineCard, { props: { statusline: active } });
		expect(screen.getByText('Active')).toBeInTheDocument();
	});

	it('should not show Active badge when inactive', () => {
		render(StatusLineCard, { props: { statusline: mockStatusLine } });
		expect(screen.queryByText('Active')).not.toBeInTheDocument();
	});

	it('should show description when present', () => {
		const withDesc = { ...mockStatusLine, description: 'A nice status line' };
		render(StatusLineCard, { props: { statusline: withDesc } });
		expect(screen.getByText('A nice status line')).toBeInTheDocument();
	});

	it('should show author when present', () => {
		const withAuthor = { ...mockStatusLine, author: 'john' };
		render(StatusLineCard, { props: { statusline: withAuthor } });
		expect(screen.getByText('by john')).toBeInTheDocument();
	});

	it('should show Activate button when inactive', () => {
		render(StatusLineCard, {
			props: { statusline: mockStatusLine, onActivate: vi.fn() }
		});
		expect(screen.getByText('Activate')).toBeInTheDocument();
	});

	it('should show Deactivate button when active', () => {
		const active = { ...mockStatusLine, isActive: true };
		render(StatusLineCard, {
			props: { statusline: active, onDeactivate: vi.fn() }
		});
		expect(screen.getByText('Deactivate')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// StatusLineForm
// ──────────────────────────────────────────────────────────
describe('StatusLineForm Component', () => {
	let StatusLineForm: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/statusline/StatusLineForm.svelte');
		StatusLineForm = mod.default;
	});

	it('should render Name label', () => {
		render(StatusLineForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Name')).toBeInTheDocument();
	});

	it('should render Description label', () => {
		render(StatusLineForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Description')).toBeInTheDocument();
	});

	it('should render Command label', () => {
		render(StatusLineForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Command')).toBeInTheDocument();
	});

	it('should render Padding label', () => {
		render(StatusLineForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Padding')).toBeInTheDocument();
	});

	it('should render Create button for new form', () => {
		render(StatusLineForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Create')).toBeInTheDocument();
	});

	it('should render Update button for existing form', () => {
		const existing = { id: 1, name: 'Existing', rawCommand: 'echo hi', statuslineType: 'raw' };
		render(StatusLineForm, {
			props: { initialValues: existing, onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByText('Update')).toBeInTheDocument();
	});

	it('should render Cancel button', () => {
		render(StatusLineForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Cancel')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// StatusLinePreview
// ──────────────────────────────────────────────────────────
describe('StatusLinePreview Component', () => {
	let StatusLinePreview: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/statusline/StatusLinePreview.svelte');
		StatusLinePreview = mod.default;
	});

	it('should render empty state when no segments', () => {
		render(StatusLinePreview, { props: { segments: [] } });
		expect(document.body.textContent).toContain('No segments');
	});

	it('should render segments in default theme', () => {
		const segments = [
			{ id: 'a', type: 'model', enabled: true, color: 'cyan', position: 0 }
		];
		render(StatusLinePreview, { props: { segments, theme: 'default' } });
		// Model preview text includes 'opus'
		expect(document.body.textContent).toContain('opus');
	});

	it('should render segments in powerline theme', () => {
		const segments = [
			{ id: 'a', type: 'model', enabled: true, color: 'cyan', position: 0 }
		];
		render(StatusLinePreview, { props: { segments, theme: 'powerline' } });
		expect(document.body.textContent).toContain('opus');
	});

	it('should render powerline_round theme', () => {
		const segments = [
			{ id: 'a', type: 'model', enabled: true, color: 'cyan', position: 0 }
		];
		render(StatusLinePreview, { props: { segments, theme: 'powerline_round' } });
		expect(document.body.textContent).toContain('opus');
	});

	it('should skip disabled segments', () => {
		const segments = [
			{ id: 'a', type: 'model', enabled: false, color: 'cyan', position: 0 },
			{ id: 'b', type: 'cost', enabled: true, color: 'green', position: 1 }
		];
		render(StatusLinePreview, { props: { segments } });
		// cost preview shows $ sign
		expect(document.body.textContent).toContain('$');
	});

	it('should render separator preview', () => {
		const segments = [
			{ id: 'a', type: 'separator', enabled: true, color: 'gray', separatorChar: '|', position: 0 }
		];
		render(StatusLinePreview, { props: { segments } });
		expect(document.body.textContent).toContain('|');
	});

	it('should render custom_text preview', () => {
		const segments = [
			{ id: 'a', type: 'custom_text', enabled: true, color: 'white', customText: 'Hello', position: 0 }
		];
		render(StatusLinePreview, { props: { segments } });
		expect(document.body.textContent).toContain('Hello');
	});

	it('should handle line_break by splitting into multiple lines', () => {
		const segments = [
			{ id: 'a', type: 'model', enabled: true, color: 'cyan', position: 0 },
			{ id: 'b', type: 'line_break', enabled: true, color: 'white', position: 1 },
			{ id: 'c', type: 'cost', enabled: true, color: 'green', position: 2 }
		];
		render(StatusLinePreview, { props: { segments } });
		expect(document.body.textContent).toContain('opus');
		expect(document.body.textContent).toContain('$');
	});

	it('should render context segment with different formats', () => {
		const segments = [
			{ id: 'a', type: 'context', enabled: true, color: 'yellow', format: 'fraction', position: 0 }
		];
		render(StatusLinePreview, { props: { segments } });
		expect(document.body.textContent).toContain('156k/200k');
	});

	it('should render context percentage format', () => {
		const segments = [
			{ id: 'a', type: 'context', enabled: true, color: 'yellow', format: 'percentage', position: 0 }
		];
		render(StatusLinePreview, { props: { segments } });
		expect(document.body.textContent).toContain('78%');
	});

	it('should render git_branch segment', () => {
		const segments = [
			{ id: 'a', type: 'git_branch', enabled: true, color: 'green', position: 0 }
		];
		render(StatusLinePreview, { props: { segments } });
		expect(document.body.textContent).toContain('main');
	});

	it('should render duration with hms format', () => {
		const segments = [
			{ id: 'a', type: 'duration', enabled: true, color: 'white', format: 'hms', position: 0 }
		];
		render(StatusLinePreview, { props: { segments } });
		expect(document.body.textContent).toContain('0:05:30');
	});

	it('should render duration with human format', () => {
		const segments = [
			{ id: 'a', type: 'duration', enabled: true, color: 'white', format: 'human', position: 0 }
		];
		render(StatusLinePreview, { props: { segments } });
		expect(document.body.textContent).toContain('5m 30s');
	});
});

// ──────────────────────────────────────────────────────────
// SegmentCard
// ──────────────────────────────────────────────────────────
describe('SegmentCard Component', () => {
	let SegmentCard: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/statusline/SegmentCard.svelte');
		SegmentCard = mod.default;
	});

	it('should render segment type label', () => {
		const seg = { id: 'a', type: 'model', enabled: true, color: 'cyan', position: 0 };
		render(SegmentCard, { props: { segment: seg } });
		// SEGMENT_TYPES label for model should appear
		expect(document.body.textContent).toContain('Model');
	});

	it('should render line_break variant', () => {
		const seg = { id: 'b', type: 'line_break', enabled: true, color: 'white', position: 0 };
		render(SegmentCard, { props: { segment: seg } });
		expect(screen.getByText('New Line')).toBeInTheDocument();
	});

	it('should render as selected', () => {
		const seg = { id: 'a', type: 'model', enabled: true, color: 'cyan', position: 0 };
		const { container } = render(SegmentCard, {
			props: { segment: seg, isSelected: true }
		});
		// The selected card has a ring class
		expect(container.innerHTML).toContain('ring-1');
	});

	it('should show opacity when disabled', () => {
		const seg = { id: 'a', type: 'model', enabled: false, color: 'cyan', position: 0 };
		const { container } = render(SegmentCard, { props: { segment: seg } });
		expect(container.innerHTML).toContain('opacity-50');
	});
});

// ──────────────────────────────────────────────────────────
// SegmentConfig
// ──────────────────────────────────────────────────────────
describe('SegmentConfig Component', () => {
	let SegmentConfig: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/statusline/SegmentConfig.svelte');
		SegmentConfig = mod.default;
	});

	it('should render Configure heading', () => {
		const seg = { id: 'a', type: 'model', enabled: true, color: 'cyan', position: 0 };
		render(SegmentConfig, { props: { segment: seg, onChange: vi.fn() } });
		expect(document.body.textContent).toContain('Configure:');
	});

	it('should render Text Color label', () => {
		const seg = { id: 'a', type: 'model', enabled: true, color: 'cyan', position: 0 };
		render(SegmentConfig, { props: { segment: seg, onChange: vi.fn() } });
		expect(screen.getByText('Text Color')).toBeInTheDocument();
	});

	it('should render Background Color for non-separator types', () => {
		const seg = { id: 'a', type: 'model', enabled: true, color: 'cyan', position: 0 };
		render(SegmentConfig, { props: { segment: seg, onChange: vi.fn() } });
		expect(screen.getByText('Background Color')).toBeInTheDocument();
	});

	it('should not render Background Color for separator type', () => {
		const seg = { id: 'a', type: 'separator', enabled: true, color: 'gray', separatorChar: '|', position: 0 };
		render(SegmentConfig, { props: { segment: seg, onChange: vi.fn() } });
		expect(screen.queryByText('Background Color')).not.toBeInTheDocument();
	});

	it('should render Label prefix for non-separator non-custom types', () => {
		const seg = { id: 'a', type: 'model', enabled: true, color: 'cyan', position: 0 };
		render(SegmentConfig, { props: { segment: seg, onChange: vi.fn() } });
		expect(screen.getByText('Label prefix')).toBeInTheDocument();
	});

	it('should render Character section for separator type', () => {
		const seg = { id: 'a', type: 'separator', enabled: true, color: 'gray', separatorChar: '|', position: 0 };
		render(SegmentConfig, { props: { segment: seg, onChange: vi.fn() } });
		expect(screen.getByText('Character')).toBeInTheDocument();
	});

	it('should render Text input for custom_text type', () => {
		const seg = { id: 'a', type: 'custom_text', enabled: true, color: 'white', customText: '', position: 0 };
		render(SegmentConfig, { props: { segment: seg, onChange: vi.fn() } });
		expect(screen.getByText('Text')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// SegmentPicker
// ──────────────────────────────────────────────────────────
describe('SegmentPicker Component', () => {
	let SegmentPicker: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/statusline/SegmentPicker.svelte');
		SegmentPicker = mod.default;
	});

	it('should render group buttons', () => {
		render(SegmentPicker, { props: { onAdd: vi.fn() } });
		expect(screen.getByText('Model & Session')).toBeInTheDocument();
		expect(screen.getByText('Usage & Cost')).toBeInTheDocument();
		expect(document.body.textContent).toContain('Git & Workspace');
	});

	it('should render Separator button', () => {
		render(SegmentPicker, { props: { onAdd: vi.fn() } });
		expect(screen.getByText('Separator')).toBeInTheDocument();
	});

	it('should render Line Break button', () => {
		render(SegmentPicker, { props: { onAdd: vi.fn() } });
		expect(screen.getByText('Line Break')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// StatusLineBuilder
// ──────────────────────────────────────────────────────────
describe('StatusLineBuilder Component', () => {
	let StatusLineBuilder: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/statusline/StatusLineBuilder.svelte');
		StatusLineBuilder = mod.default;
	});

	it('should render Name and Description labels', () => {
		render(StatusLineBuilder);
		expect(screen.getAllByText('Name').length).toBeGreaterThan(0);
		expect(screen.getAllByText('Description').length).toBeGreaterThan(0);
	});

	it('should render Add Segments heading', () => {
		render(StatusLineBuilder);
		expect(screen.getByText('Add Segments')).toBeInTheDocument();
	});

	it('should render Segments heading', () => {
		render(StatusLineBuilder);
		expect(screen.getByText('Segments (drag to reorder)')).toBeInTheDocument();
	});

	it('should render Preview heading', () => {
		render(StatusLineBuilder);
		expect(screen.getByText('Preview')).toBeInTheDocument();
	});

	it('should render theme toggle buttons', () => {
		render(StatusLineBuilder);
		expect(screen.getByText('Default')).toBeInTheDocument();
	});

	it('should render Save and Save & Activate buttons', () => {
		render(StatusLineBuilder);
		expect(screen.getByText('Save')).toBeInTheDocument();
		expect(screen.getByText('Save & Activate')).toBeInTheDocument();
	});

	it('should render default segments in preview', () => {
		render(StatusLineBuilder);
		// Default segments include model (opus) and cost ($)
		expect(document.body.textContent).toContain('opus');
	});
});

// ──────────────────────────────────────────────────────────
// StatusLineGalleryCard
// ──────────────────────────────────────────────────────────
describe('StatusLineGalleryCard Component', () => {
	let StatusLineGalleryCard: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/statusline/StatusLineGalleryCard.svelte');
		StatusLineGalleryCard = mod.default;
	});

	const mockEntry = {
		name: 'Powerline Pro',
		packageName: 'powerline-pro',
		description: 'A powerline-style status line',
		author: 'dev',
		previewText: 'opus | $1.23 | 78%',
		tags: ['powerline', 'custom'],
		homepageUrl: 'https://example.com'
	};

	it('should render entry name', () => {
		render(StatusLineGalleryCard, { props: { entry: mockEntry } });
		expect(screen.getByText('Powerline Pro')).toBeInTheDocument();
	});

	it('should render description', () => {
		render(StatusLineGalleryCard, { props: { entry: mockEntry } });
		expect(screen.getByText('A powerline-style status line')).toBeInTheDocument();
	});

	it('should render author', () => {
		render(StatusLineGalleryCard, { props: { entry: mockEntry } });
		expect(screen.getByText('by dev')).toBeInTheDocument();
	});

	it('should render preview text', () => {
		render(StatusLineGalleryCard, { props: { entry: mockEntry } });
		expect(screen.getByText('opus | $1.23 | 78%')).toBeInTheDocument();
	});

	it('should render tags', () => {
		render(StatusLineGalleryCard, { props: { entry: mockEntry } });
		expect(screen.getByText('powerline')).toBeInTheDocument();
		expect(screen.getByText('custom')).toBeInTheDocument();
	});

	it('should render Add to Library button when not installed', () => {
		render(StatusLineGalleryCard, { props: { entry: mockEntry, isInstalled: false } });
		expect(screen.getByText('Add to Library')).toBeInTheDocument();
	});

	it('should render Installed when already installed', () => {
		render(StatusLineGalleryCard, { props: { entry: mockEntry, isInstalled: true } });
		expect(screen.getByText('Installed')).toBeInTheDocument();
	});

	it('should render Installing... when installing', () => {
		render(StatusLineGalleryCard, {
			props: { entry: mockEntry, isInstalled: false, isInstalling: true }
		});
		expect(screen.getByText('Installing...')).toBeInTheDocument();
	});

	it('should not render description when absent', () => {
		const noDesc = { ...mockEntry, description: '' };
		render(StatusLineGalleryCard, { props: { entry: noDesc } });
		expect(screen.queryByText('A powerline-style status line')).not.toBeInTheDocument();
	});

	it('should not render author when absent', () => {
		const noAuthor = { ...mockEntry, author: '' };
		render(StatusLineGalleryCard, { props: { entry: noAuthor } });
		expect(screen.queryByText('by dev')).not.toBeInTheDocument();
	});

	it('should not render tags when empty', () => {
		const noTags = { ...mockEntry, tags: [] };
		render(StatusLineGalleryCard, { props: { entry: noTags } });
		expect(screen.queryByText('powerline')).not.toBeInTheDocument();
	});

	it('should not render preview when absent', () => {
		const noPreview = { ...mockEntry, previewText: '' };
		render(StatusLineGalleryCard, { props: { entry: noPreview } });
		expect(screen.queryByText('opus | $1.23 | 78%')).not.toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// StatusLineGallery
// ──────────────────────────────────────────────────────────
describe('StatusLineGallery Component', () => {
	let StatusLineGallery: any;
	let statuslineLibrary: any;

	beforeAll(async () => {
		const stores = await import('$lib/stores');
		statuslineLibrary = (stores as any).statuslineLibrary;
		const mod = await import('$lib/components/statusline/StatusLineGallery.svelte');
		StatusLineGallery = mod.default;
	});

	beforeEach(() => {
		statuslineLibrary.gallery = [];
	});

	it('should render empty state when no gallery entries', () => {
		render(StatusLineGallery);
		expect(screen.getByText('No gallery entries available')).toBeInTheDocument();
	});

	it('should render Refresh button', () => {
		render(StatusLineGallery);
		expect(screen.getByText('Refresh')).toBeInTheDocument();
	});

	it('should render gallery entries when available', () => {
		statuslineLibrary.gallery = [
			{ name: 'Gallery Entry', packageName: 'ge', description: 'Desc', author: '', previewText: '', tags: [] }
		];
		render(StatusLineGallery);
		expect(screen.getByText('Gallery Entry')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// StatusLineLibrary
// ──────────────────────────────────────────────────────────
describe('StatusLineLibrary Component', () => {
	let StatusLineLibrary: any;
	let statuslineLibrary: any;

	beforeAll(async () => {
		const stores = await import('$lib/stores');
		statuslineLibrary = (stores as any).statuslineLibrary;
		const mod = await import('$lib/components/statusline/StatusLineLibrary.svelte');
		StatusLineLibrary = mod.default;
	});

	beforeEach(() => {
		statuslineLibrary.filteredStatusLines = [];
		statuslineLibrary.isLoading = false;
		statuslineLibrary.searchQuery = '';
	});

	it('should render empty state when no status lines', () => {
		render(StatusLineLibrary);
		expect(screen.getByText('No status lines yet')).toBeInTheDocument();
	});

	it('should render search empty state', () => {
		statuslineLibrary.searchQuery = 'xyz';
		render(StatusLineLibrary);
		expect(screen.getByText('No status lines match your search')).toBeInTheDocument();
	});

	it('should render loading state', () => {
		statuslineLibrary.isLoading = true;
		render(StatusLineLibrary);
		expect(screen.getByText('Loading...')).toBeInTheDocument();
	});

	it('should render status line cards', () => {
		statuslineLibrary.filteredStatusLines = [
			{ id: 1, name: 'My SL', statuslineType: 'custom', isActive: false }
		];
		render(StatusLineLibrary);
		expect(screen.getByText('My SL')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// StatusLine index.ts exports
// ──────────────────────────────────────────────────────────
describe('StatusLine index.ts exports', () => {
	let slExports: any;

	beforeAll(async () => {
		slExports = await import('$lib/components/statusline');
	});

	it('should export all components', () => {
		expect(slExports.StatusLineLibrary).toBeDefined();
		expect(slExports.StatusLineCard).toBeDefined();
		expect(slExports.StatusLineBuilder).toBeDefined();
		expect(slExports.StatusLinePreview).toBeDefined();
		expect(slExports.StatusLineGallery).toBeDefined();
		expect(slExports.StatusLineGalleryCard).toBeDefined();
		expect(slExports.StatusLineForm).toBeDefined();
		expect(slExports.SegmentPicker).toBeDefined();
		expect(slExports.SegmentCard).toBeDefined();
		expect(slExports.SegmentConfig).toBeDefined();
	});
});
