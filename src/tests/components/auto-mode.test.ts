import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

describe('AutoModeEditor Component', () => {
	const mockSettings = {
		scope: 'user',
		availableModels: [],
		disableAutoMode: undefined,
		autoModeEnvironment: 'CI server',
		autoModeAllow: 'File reads, git operations',
		autoModeSoftDeny: 'Network access to production'
	};

	it('should render Auto Mode heading', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		render(AutoModeEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Auto Mode')).toBeInTheDocument();
	});

	it('should render Disable Auto Mode toggle', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		render(AutoModeEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Disable Auto Mode')).toBeInTheDocument();
	});

	it('should render Environment textarea', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		render(AutoModeEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByLabelText('Environment')).toBeInTheDocument();
	});

	it('should render Allow textarea', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		render(AutoModeEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByLabelText('Allow')).toBeInTheDocument();
	});

	it('should render Soft Deny textarea', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		render(AutoModeEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByLabelText('Soft Deny')).toBeInTheDocument();
	});

	it('should populate fields from settings', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		render(AutoModeEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		const envTextarea = screen.getByLabelText('Environment') as HTMLTextAreaElement;
		expect(envTextarea.value).toBe('CI server');
	});

	it('should call onsave when save clicked', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		const onsave = vi.fn();
		render(AutoModeEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Auto Mode Settings'));
		expect(onsave).toHaveBeenCalledOnce();
	});

	it('should populate all textarea values from settings', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		render(AutoModeEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect((screen.getByLabelText('Environment') as HTMLTextAreaElement).value).toBe('CI server');
		expect((screen.getByLabelText('Allow') as HTMLTextAreaElement).value).toBe('File reads, git operations');
		expect((screen.getByLabelText('Soft Deny') as HTMLTextAreaElement).value).toBe('Network access to production');
	});

	it('should save with correct field values', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		const onsave = vi.fn();
		render(AutoModeEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Auto Mode Settings'));
		const saved = onsave.mock.calls[0][0];
		expect(saved.autoModeEnvironment).toBe('CI server');
		expect(saved.autoModeAllow).toBe('File reads, git operations');
		expect(saved.autoModeSoftDeny).toBe('Network access to production');
	});

	it('should save with disableAutoMode when toggle checked', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		const onsave = vi.fn();
		const settingsWithDisable = { ...mockSettings, disableAutoMode: true };
		render(AutoModeEditor, {
			props: { settings: settingsWithDisable as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Auto Mode Settings'));
		expect(onsave.mock.calls[0][0].disableAutoMode).toBe(true);
	});

	it('should handle toggle interaction', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		const onsave = vi.fn();
		render(AutoModeEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		const checkbox = screen.getByLabelText('Disable Auto Mode') as HTMLInputElement;
		await fireEvent.click(checkbox);
		await fireEvent.click(screen.getByText('Save Auto Mode Settings'));
		expect(onsave.mock.calls[0][0].disableAutoMode).toBe(true);
	});

	it('should save partial updates (only one field set)', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		const onsave = vi.fn();
		const partialSettings = { ...mockSettings, autoModeEnvironment: 'staging', autoModeAllow: '', autoModeSoftDeny: '' };
		render(AutoModeEditor, {
			props: { settings: partialSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Auto Mode Settings'));
		const saved = onsave.mock.calls[0][0];
		expect(saved.autoModeEnvironment).toBe('staging');
		expect(saved.autoModeAllow).toBeUndefined();
		expect(saved.autoModeSoftDeny).toBeUndefined();
	});

	it('should pass undefined for empty string fields', async () => {
		const { default: AutoModeEditor } = await import('$lib/components/auto-mode/AutoModeEditor.svelte');
		const onsave = vi.fn();
		const emptySettings = { ...mockSettings, autoModeEnvironment: '', autoModeAllow: '', autoModeSoftDeny: '' };
		render(AutoModeEditor, {
			props: { settings: emptySettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Auto Mode Settings'));
		const savedSettings = onsave.mock.calls[0][0];
		expect(savedSettings.autoModeEnvironment).toBeUndefined();
		expect(savedSettings.autoModeAllow).toBeUndefined();
		expect(savedSettings.autoModeSoftDeny).toBeUndefined();
	});
});

describe('Auto-mode index.ts exports', () => {
	it('should export AutoModeEditor', async () => {
		const exports = await import('$lib/components/auto-mode');
		expect(exports.AutoModeEditor).toBeDefined();
	});
});
