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

	it('should render Memory & Instructions heading', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		render(SessionCleanupEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Memory & Instructions')).toBeInTheDocument();
	});

	it('should render Teammate Mode dropdown in Agent Teams card', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		render(SessionCleanupEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByLabelText('Teammate Mode')).toBeInTheDocument();
	});

	it('should render Plans Directory input', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		render(SessionCleanupEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		const input = screen.getByLabelText('Plans Directory') as HTMLInputElement;
		expect(input.value).toBe('./plans');
	});

	it('should render Default Agent input', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		render(SessionCleanupEditor, {
			props: { settings: { ...mockSettings, agent: 'code-reviewer' } as any, onsave: vi.fn() }
		});
		const input = screen.getByLabelText('Default Agent') as HTMLInputElement;
		expect(input.value).toBe('code-reviewer');
	});

	it('should include agentTeamEnabled in save payload', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		const onsave = vi.fn();
		const settingsWithTeams = { ...mockSettings, agentTeamEnabled: true };
		render(SessionCleanupEditor, {
			props: { settings: settingsWithTeams as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Session Settings'));
		expect(onsave.mock.calls[0][0].agentTeamEnabled).toBe(true);
	});

	it('should toggle agent teams on', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		const onsave = vi.fn();
		render(SessionCleanupEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		const checkbox = screen.getByLabelText('Enable Agent Teams') as HTMLInputElement;
		await fireEvent.click(checkbox);
		await fireEvent.click(screen.getByText('Save Session Settings'));
		expect(onsave.mock.calls[0][0].agentTeamEnabled).toBe(true);
	});

	it('should include teammateMode in save payload', async () => {
		const { default: SessionCleanupEditor } = await import('$lib/components/session-cleanup/SessionCleanupEditor.svelte');
		const onsave = vi.fn();
		render(SessionCleanupEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Session Settings'));
		expect(onsave.mock.calls[0][0].teammateMode).toBe('full');
	});
});

describe('Session-cleanup index.ts exports', () => {
	it('should export SessionCleanupEditor', async () => {
		const exports = await import('$lib/components/session-cleanup');
		expect(exports.SessionCleanupEditor).toBeDefined();
	});
});
