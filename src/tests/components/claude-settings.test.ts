import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/types', async (importOriginal) => {
	const actual = await importOriginal() as any;
	return {
		...actual,
		CLAUDE_MODELS: actual.CLAUDE_MODELS ?? [
			{ value: 'sonnet', label: 'Sonnet', description: 'Balanced' }
		],
		AVAILABLE_MODEL_SHORTCUTS: actual.AVAILABLE_MODEL_SHORTCUTS ?? [
			{ value: 'sonnet', label: 'Sonnet' }
		],
		OUTPUT_STYLES: actual.OUTPUT_STYLES ?? [
			{ value: '', label: 'Default' },
			{ value: 'concise', label: 'Concise' }
		],
		COMMON_LANGUAGES: actual.COMMON_LANGUAGES ?? [
			{ value: '', label: 'Default' },
			{ value: 'en', label: 'English' }
		]
	};
});

describe('ModelConfigEditor Component', () => {
	let ModelConfigEditor: any;

	const mockSettings = {
		scope: 'project',
		availableModels: ['sonnet'],
		model: 'sonnet',
		outputStyle: 'concise',
		language: 'en',
		alwaysThinkingEnabled: true
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/claude-settings/ModelConfigEditor.svelte');
		ModelConfigEditor = mod.default;
	});

	it('should render Model & Output heading', () => {
		render(ModelConfigEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Model & Output')).toBeInTheDocument();
	});

	it('should render model select', () => {
		render(ModelConfigEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByLabelText('Default Model')).toBeInTheDocument();
	});

	it('should render output style select', () => {
		render(ModelConfigEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByLabelText('Output Style')).toBeInTheDocument();
	});

	it('should render language select', () => {
		render(ModelConfigEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByLabelText('Response Language')).toBeInTheDocument();
	});

	it('should call onsave when save clicked', async () => {
		const onsave = vi.fn();
		render(ModelConfigEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Model Settings'));
		expect(onsave).toHaveBeenCalledOnce();
	});
});

describe('AttributionEditor Component', () => {
	let AttributionEditor: any;

	const mockSettings = {
		scope: 'project',
		availableModels: [],
		attributionCommit: undefined,
		attributionPr: undefined
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/claude-settings/AttributionEditor.svelte');
		AttributionEditor = mod.default;
	});

	it('should render Attribution heading', () => {
		render(AttributionEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Attribution')).toBeInTheDocument();
	});

	it('should show default text when not set', () => {
		render(AttributionEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getAllByText('Set to empty (hide)').length).toBe(2);
	});

	it('should show textarea when commit attribution is set', () => {
		const settingsWithCommit = { ...mockSettings, attributionCommit: 'Custom commit text' };
		render(AttributionEditor, {
			props: { settings: settingsWithCommit as any, onsave: vi.fn() }
		});
		const textarea = screen.getByLabelText('Commit Attribution') as HTMLTextAreaElement;
		expect(textarea.value).toBe('Custom commit text');
	});

	it('should call onsave when save clicked', async () => {
		const onsave = vi.fn();
		render(AttributionEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Attribution'));
		expect(onsave).toHaveBeenCalledOnce();
	});
});

describe('Model aliases (CLAUDE_MODELS)', () => {
	it('should expose the core Anthropic aliases', async () => {
		const { CLAUDE_MODELS } = await import('$lib/types');
		const values = CLAUDE_MODELS.map(m => m.value);
		expect(values).toContain('opus');
		expect(values).toContain('sonnet');
		expect(values).toContain('haiku');
	});

	it('should mark Opus and Sonnet as 1M-capable, Haiku as not', async () => {
		const { CLAUDE_MODELS } = await import('$lib/types');
		const byValue = Object.fromEntries(CLAUDE_MODELS.map(m => [m.value, m]));
		expect(byValue.opus.supports1m).toBe(true);
		expect(byValue.sonnet.supports1m).toBe(true);
		expect(byValue.haiku.supports1m).toBe(false);
	});

	it('should include extended context shortcuts in AVAILABLE_MODEL_SHORTCUTS', async () => {
		const { AVAILABLE_MODEL_SHORTCUTS } = await import('$lib/types');
		const values = AVAILABLE_MODEL_SHORTCUTS.map(s => s.value);
		expect(values).toContain('sonnet');
		expect(values).toContain('opus');
		expect(values).toContain('haiku');
	});
});

describe('Claude-settings index.ts exports', () => {
	let csExports: any;

	beforeAll(async () => {
		csExports = await import('$lib/components/claude-settings');
	});

	it('should export all components', () => {
		expect(csExports.ModelConfigEditor).toBeDefined();
		expect(csExports.AttributionEditor).toBeDefined();
		expect(csExports.ModelOverridesEditor).toBeDefined();
	});
});
