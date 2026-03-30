import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/types', async (importOriginal) => {
	const actual = await importOriginal() as any;
	return {
		...actual,
		AUTO_UPDATES_CHANNELS: actual.AUTO_UPDATES_CHANNELS ?? [
			{ value: '', label: 'Default' },
			{ value: 'stable', label: 'Stable' },
			{ value: 'beta', label: 'Beta' }
		],
		TEAMMATE_MODES: actual.TEAMMATE_MODES ?? [
			{ value: '', label: 'Default' },
			{ value: 'full', label: 'Full' },
			{ value: 'limited', label: 'Limited' }
		]
	};
});

describe('SessionCleanupEditor Component', () => {
	const mockSettings = {
		scope: 'project',
		availableModels: [],
		cleanupPeriodDays: 30,
		autoUpdatesChannel: 'stable',
		teammateMode: 'full',
		plansDirectory: './plans'
	};

	it('should render heading', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		render(SessionCleanupEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Session & Cleanup')).toBeInTheDocument();
	});

	it('should populate cleanup days', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		render(SessionCleanupEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		const input = screen.getByLabelText('Cleanup Period (days)') as HTMLInputElement;
		expect(input.value).toBe('30');
	});

	it('should call onsave on save click', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		const onsave = vi.fn();
		render(SessionCleanupEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Session Settings'));
		expect(onsave).toHaveBeenCalledOnce();
	});

	it('should render Agent Teams heading', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		render(SessionCleanupEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Agent Teams')).toBeInTheDocument();
	});

	it('should render Enable Agent Teams toggle', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		render(SessionCleanupEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Enable Agent Teams')).toBeInTheDocument();
	});

	it('should not render Disable Auto Mode (moved to Auto Mode tab)', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		render(SessionCleanupEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.queryByText('Disable Auto Mode')).not.toBeInTheDocument();
	});
});

describe('Session-cleanup index.ts exports', () => {
	it('should export SessionCleanupEditor', async () => {
		const exports = await import('$lib/components/session-cleanup');
		expect(exports.SessionCleanupEditor).toBeDefined();
	});
});
