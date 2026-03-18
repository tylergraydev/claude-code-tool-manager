import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	commandLibrary: {
		commands: [],
		filteredCommands: [],
		isLoading: false,
		searchQuery: '',
		updateCommand: vi.fn(),
		getCommandById: vi.fn()
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
	invoke: vi.fn()
}));

describe('CommandCard Component', () => {
	const mockCommand = {
		id: 1,
		name: 'test-cmd',
		description: 'A test command',
		content: 'Do the thing',
		allowedTools: ['Read', 'Edit'],
		argumentHint: '[file]',
		tags: ['utility', 'git', 'deploy'],
		source: 'user' as const,
		isFavorite: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	};

	it('should render command name with slash prefix', async () => {
		const { default: CommandCard } = await import('$lib/components/commands/CommandCard.svelte');
		render(CommandCard, { props: { command: mockCommand } });
		expect(screen.getByText('/test-cmd')).toBeInTheDocument();
	});

	it('should render description', async () => {
		const { default: CommandCard } = await import('$lib/components/commands/CommandCard.svelte');
		render(CommandCard, { props: { command: mockCommand } });
		expect(screen.getByText('A test command')).toBeInTheDocument();
	});

	it('should show tool count badge', async () => {
		const { default: CommandCard } = await import('$lib/components/commands/CommandCard.svelte');
		render(CommandCard, { props: { command: mockCommand } });
		expect(screen.getByText('2 tools')).toBeInTheDocument();
	});

	it('should show argument hint', async () => {
		const { default: CommandCard } = await import('$lib/components/commands/CommandCard.svelte');
		render(CommandCard, { props: { command: mockCommand } });
		expect(screen.getByText('[file]')).toBeInTheDocument();
	});

	it('should show tags with overflow indicator', async () => {
		const { default: CommandCard } = await import('$lib/components/commands/CommandCard.svelte');
		render(CommandCard, { props: { command: mockCommand } });
		expect(screen.getByText('utility')).toBeInTheDocument();
		expect(screen.getByText('git')).toBeInTheDocument();
		expect(screen.getByText('+1')).toBeInTheDocument();
	});

	it('should show auto-detected badge', async () => {
		const { default: CommandCard } = await import('$lib/components/commands/CommandCard.svelte');
		render(CommandCard, {
			props: { command: { ...mockCommand, source: 'auto-detected' } }
		});
		expect(screen.getByText('Auto')).toBeInTheDocument();
	});

	it('should hide actions when showActions is false', async () => {
		const { default: CommandCard } = await import('$lib/components/commands/CommandCard.svelte');
		render(CommandCard, { props: { command: mockCommand, showActions: false } });
		expect(screen.queryByLabelText(/Actions for/)).not.toBeInTheDocument();
	});
});

describe('CommandForm Component', () => {
	it('should render the form with required fields', async () => {
		const { default: CommandForm } = await import('$lib/components/commands/CommandForm.svelte');
		render(CommandForm, {
			props: { onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByLabelText(/Name/)).toBeInTheDocument();
		expect(screen.getByLabelText(/Command Prompt/)).toBeInTheDocument();
	});

	it('should show Create Command button for new command', async () => {
		const { default: CommandForm } = await import('$lib/components/commands/CommandForm.svelte');
		render(CommandForm, {
			props: { onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByText('Create Command')).toBeInTheDocument();
	});

	it('should show Update Command button when editing', async () => {
		const { default: CommandForm } = await import('$lib/components/commands/CommandForm.svelte');
		render(CommandForm, {
			props: { initialValues: { name: 'existing' }, onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByText('Update Command')).toBeInTheDocument();
	});

	it('should call onCancel when cancel clicked', async () => {
		const { default: CommandForm } = await import('$lib/components/commands/CommandForm.svelte');
		const onCancel = vi.fn();
		render(CommandForm, {
			props: { onSubmit: vi.fn(), onCancel }
		});
		await fireEvent.click(screen.getByText('Cancel'));
		expect(onCancel).toHaveBeenCalledOnce();
	});
});

describe('CommandLibrary Component', () => {
	it('should render empty state when no commands', async () => {
		const { default: CommandLibrary } = await import('$lib/components/commands/CommandLibrary.svelte');
		render(CommandLibrary, { props: {} });
		expect(screen.getByText('No commands in library')).toBeInTheDocument();
	});
});

describe('Commands index.ts exports', () => {
	it('should export all command components', async () => {
		const exports = await import('$lib/components/commands');
		expect(exports.CommandCard).toBeDefined();
		expect(exports.CommandLibrary).toBeDefined();
		expect(exports.CommandForm).toBeDefined();
	});
});
