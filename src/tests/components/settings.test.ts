import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	claudeSettingsLibrary: {
		isLoading: false,
		error: null,
		selectedScope: 'user',
		currentScopeSettings: null,
		load: vi.fn(),
		save: vi.fn(),
		setScope: vi.fn()
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	},
	keybindingsLibrary: {
		isLoading: false,
		bindings: {},
		mergedBindings: [],
		setBinding: vi.fn(),
		unbindKey: vi.fn(),
		resetContext: vi.fn(),
		save: vi.fn(),
		load: vi.fn()
	},
	spinnerVerbLibrary: {
		verbs: [],
		isLoading: false,
		load: vi.fn()
	},
	projectsStore: {
		projects: []
	}
}));

vi.mock('$lib/types', async (importOriginal) => {
	const actual = await importOriginal() as any;
	return {
		...actual,
		CLAUDE_SETTINGS_SCOPE_LABELS: actual.CLAUDE_SETTINGS_SCOPE_LABELS ?? {
			user: { label: 'User', description: 'User scope' },
			project: { label: 'Project', description: 'Project scope' },
			local: { label: 'Local', description: 'Local scope' }
		}
	};
});

describe('settingsCategories', () => {
	it('should export SETTINGS_CATEGORIES array', async () => {
		const { SETTINGS_CATEGORIES } = await import('$lib/components/settings/settingsCategories');
		expect(Array.isArray(SETTINGS_CATEGORIES)).toBe(true);
		expect(SETTINGS_CATEGORIES.length).toBeGreaterThan(0);
		expect(SETTINGS_CATEGORIES[0]).toHaveProperty('id');
		expect(SETTINGS_CATEGORIES[0]).toHaveProperty('label');
		expect(SETTINGS_CATEGORIES[0]).toHaveProperty('icon');
		expect(SETTINGS_CATEGORIES[0]).toHaveProperty('type');
	});

	it('should have both scoped and standalone categories', async () => {
		const { SETTINGS_CATEGORIES } = await import('$lib/components/settings/settingsCategories');
		const scoped = SETTINGS_CATEGORIES.filter(c => c.type === 'scoped');
		const standalone = SETTINGS_CATEGORIES.filter(c => c.type === 'standalone');
		expect(scoped.length).toBeGreaterThan(0);
		expect(standalone.length).toBeGreaterThan(0);
	});
});

describe('ScopedSettingsWrapper Component', () => {
	it('should be importable', async () => {
		const { default: ScopedSettingsWrapper } = await import('$lib/components/settings/ScopedSettingsWrapper.svelte');
		expect(ScopedSettingsWrapper).toBeDefined();
	});
});

describe('Settings index.ts exports', () => {
	it('should export all settings components', async () => {
		const exports = await import('$lib/components/settings');
		expect(exports.ScopedSettingsWrapper).toBeDefined();
		expect(exports.SETTINGS_CATEGORIES).toBeDefined();
	});
});
