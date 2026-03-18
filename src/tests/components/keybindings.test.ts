import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	keybindingsLibrary: {
		isLoading: false,
		bindings: {},
		mergedBindings: [],
		filteredByContext: new Map(),
		expandedContexts: new Set(),
		searchQuery: '',
		hasOverrides: false,
		overrideCount: 0,
		overrides: [],
		setBinding: vi.fn(),
		unbindKey: vi.fn(),
		removeOverride: vi.fn(),
		resetContext: vi.fn(),
		resetAll: vi.fn(),
		toggleContext: vi.fn(),
		expandAll: vi.fn(),
		collapseAll: vi.fn(),
		save: vi.fn(),
		load: vi.fn(),
		isReservedKey: vi.fn().mockReturnValue(false),
		isTerminalConflict: vi.fn().mockReturnValue(false),
		detectConflicts: vi.fn().mockReturnValue([])
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

vi.mock('$lib/types', async (importOriginal) => {
	const actual = await importOriginal() as any;
	return {
		...actual,
		KEYBINDING_CONTEXTS: actual.KEYBINDING_CONTEXTS ?? [
			{ context: 'global', label: 'Global', description: 'Active everywhere' }
		],
		RESERVED_KEYS: actual.RESERVED_KEYS ?? new Set(['ctrl+c']),
		TERMINAL_CONFLICT_KEYS: actual.TERMINAL_CONFLICT_KEYS ?? new Set([]),
		formatKeystroke: actual.formatKeystroke ?? ((k: string) => k)
	};
});

describe('KeyCaptureDialog Component', () => {
	let KeyCaptureDialog: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/keybindings/KeyCaptureDialog.svelte');
		KeyCaptureDialog = mod.default;
	});

	it('should render Capture Keybinding heading', () => {
		render(KeyCaptureDialog, {
			props: {
				context: 'global' as any,
				action: 'test-action',
				actionLabel: 'Test Action',
				currentKeys: [],
				onconfirm: vi.fn(),
				oncancel: vi.fn()
			}
		});
		expect(screen.getByText('Capture Keybinding')).toBeInTheDocument();
	});

	it('should render action label', () => {
		render(KeyCaptureDialog, {
			props: {
				context: 'global' as any,
				action: 'test-action',
				actionLabel: 'Test Action',
				currentKeys: [],
				onconfirm: vi.fn(),
				oncancel: vi.fn()
			}
		});
		expect(screen.getByText('Test Action')).toBeInTheDocument();
	});

	it('should render context name', () => {
		render(KeyCaptureDialog, {
			props: {
				context: 'global' as any,
				action: 'test-action',
				actionLabel: 'Test Action',
				currentKeys: [],
				onconfirm: vi.fn(),
				oncancel: vi.fn()
			}
		});
		expect(screen.getByText('global')).toBeInTheDocument();
	});

	it('should show press key instructions when no key captured', () => {
		render(KeyCaptureDialog, {
			props: {
				context: 'global' as any,
				action: 'test-action',
				actionLabel: 'Test Action',
				currentKeys: [],
				onconfirm: vi.fn(),
				oncancel: vi.fn()
			}
		});
		expect(screen.getByText('Press a key combination...')).toBeInTheDocument();
	});

	it('should show current keys when provided', () => {
		render(KeyCaptureDialog, {
			props: {
				context: 'global' as any,
				action: 'test-action',
				actionLabel: 'Test Action',
				currentKeys: ['ctrl+a'],
				onconfirm: vi.fn(),
				oncancel: vi.fn()
			}
		});
		expect(screen.getByText('Currently bound:')).toBeInTheDocument();
	});

	it('should render Cancel and Confirm buttons', () => {
		render(KeyCaptureDialog, {
			props: {
				context: 'global' as any,
				action: 'test-action',
				actionLabel: 'Test Action',
				currentKeys: [],
				onconfirm: vi.fn(),
				oncancel: vi.fn()
			}
		});
		expect(screen.getByText('Cancel')).toBeInTheDocument();
		expect(screen.getByText('Confirm')).toBeInTheDocument();
	});

	it('should have Confirm button disabled when no key captured', () => {
		render(KeyCaptureDialog, {
			props: {
				context: 'global' as any,
				action: 'test-action',
				actionLabel: 'Test Action',
				currentKeys: [],
				onconfirm: vi.fn(),
				oncancel: vi.fn()
			}
		});
		expect(screen.getByText('Confirm')).toBeDisabled();
	});

	it('should render Clear button', () => {
		render(KeyCaptureDialog, {
			props: {
				context: 'global' as any,
				action: 'test-action',
				actionLabel: 'Test Action',
				currentKeys: [],
				onconfirm: vi.fn(),
				oncancel: vi.fn()
			}
		});
		expect(screen.getByText('Clear')).toBeInTheDocument();
	});
});

describe('KeybindingsEditor Component', () => {
	let KeybindingsEditor: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/keybindings/KeybindingsEditor.svelte');
		KeybindingsEditor = mod.default;
	});

	it('should render keybindings editor', () => {
		render(KeybindingsEditor);
		expect(document.body).toBeTruthy();
	});

	it('should render search field', () => {
		render(KeybindingsEditor);
		expect(screen.getByPlaceholderText('Search actions, keys...')).toBeInTheDocument();
	});

	it('should show Expand All and Collapse All buttons', () => {
		render(KeybindingsEditor);
		expect(screen.getByText('Expand All')).toBeInTheDocument();
		expect(screen.getByText('Collapse All')).toBeInTheDocument();
	});

	it('should show Save Keybindings button', () => {
		render(KeybindingsEditor);
		expect(screen.getByText('Save Keybindings')).toBeInTheDocument();
	});
});

describe('Keybindings index.ts exports', () => {
	let kbExports: any;

	beforeAll(async () => {
		kbExports = await import('$lib/components/keybindings');
	});

	it('should export all components', () => {
		expect(kbExports.KeybindingsEditor).toBeDefined();
		expect(kbExports.KeyCaptureDialog).toBeDefined();
	});
});
