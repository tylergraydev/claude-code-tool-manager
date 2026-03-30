import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';

// Mock stores
vi.mock('$lib/stores', () => ({
	notifications: {
		notifications: [],
		remove: vi.fn(),
		success: vi.fn(),
		error: vi.fn(),
		info: vi.fn(),
		warning: vi.fn()
	},
	updater: {
		status: 'idle',
		update: null,
		error: null,
		downloadProgress: 0,
		checkForUpdates: vi.fn(),
		downloadAndInstall: vi.fn(),
		restartApp: vi.fn(),
		dismiss: vi.fn()
	}
}));

vi.mock('$lib/stores/updater.svelte', () => ({
	updater: {
		status: 'idle',
		update: null,
		error: null,
		downloadProgress: 0,
		checkForUpdates: vi.fn(),
		downloadAndInstall: vi.fn(),
		restartApp: vi.fn(),
		dismiss: vi.fn()
	}
}));

vi.mock('$lib/stores/whatsNew.svelte', () => ({
	whatsNew: {
		isOpen: false,
		isLoading: false,
		release: null,
		dismiss: vi.fn(),
		check: vi.fn()
	}
}));

vi.mock('@tauri-apps/plugin-shell', () => ({
	open: vi.fn()
}));

describe('Badge Component', () => {
	let Badge: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/shared/Badge.svelte');
		Badge = mod.default;
	});

	it('should render with default variant', () => {
		const children = createRawSnippet(() => ({ render: () => '<span>Test Badge</span>' }));
		render(Badge, { props: { children } });
		expect(screen.getByText('Test Badge')).toBeInTheDocument();
	});

	it('should apply success variant classes', () => {
		const children = createRawSnippet(() => ({ render: () => '<span>Success</span>' }));
		const { container } = render(Badge, { props: { variant: 'success', children } });
		const badge = container.querySelector('span.inline-flex');
		expect(badge?.className).toContain('bg-green-100');
	});

	it('should show cursor-help when title is provided', () => {
		const children = createRawSnippet(() => ({ render: () => '<span>Info</span>' }));
		const { container } = render(Badge, { props: { title: 'Help text', children } });
		const badge = container.querySelector('span.inline-flex');
		expect(badge?.getAttribute('title')).toBe('Help text');
		expect(badge?.className).toContain('cursor-help');
	});
});

describe('LoadingSpinner Component', () => {
	let LoadingSpinner: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/shared/LoadingSpinner.svelte');
		LoadingSpinner = mod.default;
	});

	it('should render with default md size', () => {
		render(LoadingSpinner);
		const spinner = screen.getByRole('status');
		expect(spinner).toBeInTheDocument();
		expect(spinner).toHaveAttribute('aria-label', 'Loading');
	});

	it('should render with sm size', () => {
		const { container } = render(LoadingSpinner, { props: { size: 'sm' } });
		const inner = container.querySelector('.animate-spin');
		expect(inner?.className).toContain('h-5');
	});

	it('should render with lg size', () => {
		const { container } = render(LoadingSpinner, { props: { size: 'lg' } });
		const inner = container.querySelector('.animate-spin');
		expect(inner?.className).toContain('h-12');
	});
});

describe('EmptyState Component', () => {
	let EmptyState: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/shared/EmptyState.svelte');
		EmptyState = mod.default;
	});

	it('should render title and description', () => {
		const MockIcon = vi.fn();
		render(EmptyState, {
			props: {
				icon: MockIcon,
				title: 'Nothing here',
				description: 'Add something to get started'
			}
		});
		expect(screen.getByText('Nothing here')).toBeInTheDocument();
		expect(screen.getByText('Add something to get started')).toBeInTheDocument();
	});

	it('should render without description', () => {
		const MockIcon = vi.fn();
		render(EmptyState, {
			props: { icon: MockIcon, title: 'Empty' }
		});
		expect(screen.getByText('Empty')).toBeInTheDocument();
	});

	it('should render children snippet', () => {
		const MockIcon = vi.fn();
		const children = createRawSnippet(() => ({ render: () => '<button>Add Item</button>' }));
		render(EmptyState, {
			props: { icon: MockIcon, title: 'Empty', children }
		});
		expect(screen.getByText('Add Item')).toBeInTheDocument();
	});
});

describe('ActionMenuItem Component', () => {
	let ActionMenuItem: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/shared/ActionMenuItem.svelte');
		ActionMenuItem = mod.default;
	});

	it('should render with default variant', () => {
		const MockIcon = vi.fn();
		render(ActionMenuItem, {
			props: { icon: MockIcon, label: 'Edit', onclick: vi.fn() }
		});
		const btn = screen.getByRole('menuitem');
		expect(btn).toBeInTheDocument();
		expect(screen.getByText('Edit')).toBeInTheDocument();
		expect(btn.className).toContain('text-gray-700');
	});

	it('should render with danger variant', () => {
		const MockIcon = vi.fn();
		render(ActionMenuItem, {
			props: { icon: MockIcon, label: 'Delete', variant: 'danger', onclick: vi.fn() }
		});
		const btn = screen.getByRole('menuitem');
		expect(btn.className).toContain('text-red-600');
	});

	it('should call onclick handler', async () => {
		const MockIcon = vi.fn();
		const onclick = vi.fn();
		render(ActionMenuItem, {
			props: { icon: MockIcon, label: 'Action', onclick }
		});
		await fireEvent.click(screen.getByRole('menuitem'));
		expect(onclick).toHaveBeenCalledOnce();
	});
});

describe('ActionMenu Component', () => {
	let ActionMenu: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/shared/ActionMenu.svelte');
		ActionMenu = mod.default;
	});

	it('should render trigger button with label', () => {
		const children = createRawSnippet(() => ({ render: () => '<button role="menuitem">Item</button>' }));
		render(ActionMenu, {
			props: { label: 'Menu for test', children }
		});
		const trigger = screen.getByLabelText('Menu for test');
		expect(trigger).toBeInTheDocument();
		expect(trigger).toHaveAttribute('aria-haspopup', 'menu');
		expect(trigger).toHaveAttribute('aria-expanded', 'false');
	});

	it('should toggle menu on click', async () => {
		const children = createRawSnippet(() => ({ render: () => '<button role="menuitem">Item</button>' }));
		render(ActionMenu, {
			props: { label: 'Test menu', children }
		});
		const trigger = screen.getByLabelText('Test menu');
		await fireEvent.click(trigger);
		expect(trigger).toHaveAttribute('aria-expanded', 'true');
		expect(screen.getByRole('menu')).toBeInTheDocument();
	});
});

describe('SearchBar Component', () => {
	let SearchBar: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/shared/SearchBar.svelte');
		SearchBar = mod.default;
	});

	it('should render with default placeholder', () => {
		render(SearchBar, { props: { value: '' } });
		expect(screen.getByPlaceholderText('Search...')).toBeInTheDocument();
	});

	it('should render with custom placeholder', () => {
		render(SearchBar, { props: { value: '', placeholder: 'Find items...' } });
		expect(screen.getByPlaceholderText('Find items...')).toBeInTheDocument();
	});

	it('should show clear button when value is non-empty', () => {
		const { container } = render(SearchBar, { props: { value: 'test' } });
		const buttons = container.querySelectorAll('button');
		expect(buttons.length).toBe(1);
	});

	it('should not show clear button when value is empty', () => {
		const { container } = render(SearchBar, { props: { value: '' } });
		const buttons = container.querySelectorAll('button');
		expect(buttons.length).toBe(0);
	});
});

describe('Toast Component', () => {
	let Toast: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/shared/Toast.svelte');
		Toast = mod.default;
	});

	it('should render notification messages', async () => {
		const { notifications } = await import('$lib/stores');
		(notifications as any).notifications = [
			{ id: '1', type: 'success', message: 'Operation completed' }
		];
		render(Toast);
		const container = screen.getByRole('status');
		expect(container).toBeInTheDocument();
		(notifications as any).notifications = [];
	});
});

describe('UpdateNotification Component', () => {
	let UpdateNotification: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/shared/UpdateNotification.svelte');
		UpdateNotification = mod.default;
	});

	it('should render without errors when status is idle', () => {
		render(UpdateNotification);
		expect(document.body).toBeTruthy();
	});

	it('should show Update Available when status is available', async () => {
		const { updater } = await import('$lib/stores/updater.svelte');
		(updater as any).status = 'available';
		(updater as any).update = { version: '4.0.0' };
		render(UpdateNotification);
		expect(screen.getByText('Update Available')).toBeInTheDocument();
		expect(screen.getByText(/Version 4.0.0/)).toBeInTheDocument();
		expect(screen.getByText('Download')).toBeInTheDocument();
		expect(screen.getByText('Later')).toBeInTheDocument();
		(updater as any).status = 'idle';
		(updater as any).update = null;
	});

	it('should show downloading state', async () => {
		const { updater } = await import('$lib/stores/updater.svelte');
		(updater as any).status = 'downloading';
		(updater as any).downloadProgress = 50;
		render(UpdateNotification);
		expect(screen.getByText('Downloading Update...')).toBeInTheDocument();
		(updater as any).status = 'idle';
	});

	it('should show ready state with restart button', async () => {
		const { updater } = await import('$lib/stores/updater.svelte');
		(updater as any).status = 'ready';
		render(UpdateNotification);
		expect(screen.getByText('Update Ready')).toBeInTheDocument();
		expect(screen.getByText('Restart Now')).toBeInTheDocument();
		(updater as any).status = 'idle';
	});

	it('should show error state', async () => {
		const { updater } = await import('$lib/stores/updater.svelte');
		(updater as any).status = 'error';
		(updater as any).error = 'Network error';
		render(UpdateNotification);
		expect(screen.getByText('Update Error')).toBeInTheDocument();
		expect(screen.getByText('Network error')).toBeInTheDocument();
		expect(screen.getByText('Dismiss')).toBeInTheDocument();
		(updater as any).status = 'idle';
		(updater as any).error = null;
	});
});

describe('WhatsNewModal Component', () => {
	let WhatsNewModal: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/shared/WhatsNewModal.svelte');
		WhatsNewModal = mod.default;
	});

	it('should not render when not open', () => {
		render(WhatsNewModal);
		expect(screen.queryByText("What's New")).not.toBeInTheDocument();
	});

	it('should render modal when open', async () => {
		const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
		(whatsNew as any).isOpen = true;
		(whatsNew as any).release = {
			version: '3.2.0',
			body: '- Bug fixes\n- New features',
			htmlUrl: 'https://github.com/test',
			publishedAt: '2024-06-01'
		};
		render(WhatsNewModal);
		expect(screen.getByText("What's New")).toBeInTheDocument();
		expect(screen.getByText('Version 3.2.0')).toBeInTheDocument();
		expect(screen.getByText('Got it')).toBeInTheDocument();
		expect(screen.getByText('View on GitHub')).toBeInTheDocument();
		(whatsNew as any).isOpen = false;
		(whatsNew as any).release = null;
	});

	it('should show loading state', async () => {
		const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
		(whatsNew as any).isOpen = true;
		(whatsNew as any).isLoading = true;
		(whatsNew as any).release = null;
		render(WhatsNewModal);
		// Loading spinner should be present
		const spinner = document.querySelector('.animate-spin');
		expect(spinner).toBeInTheDocument();
		(whatsNew as any).isOpen = false;
		(whatsNew as any).isLoading = false;
	});

	it('should show no release notes when release is null', async () => {
		const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
		(whatsNew as any).isOpen = true;
		(whatsNew as any).isLoading = false;
		(whatsNew as any).release = null;
		render(WhatsNewModal);
		expect(screen.getByText('No release notes available.')).toBeInTheDocument();
		(whatsNew as any).isOpen = false;
	});
});

describe('Shared index.ts exports', () => {
	let sharedExports: any;

	beforeAll(async () => {
		sharedExports = await import('$lib/components/shared');
	});

	it('should export all shared components', () => {
		expect(sharedExports.Toast).toBeDefined();
		expect(sharedExports.EnvEditor).toBeDefined();
		expect(sharedExports.ConfirmDialog).toBeDefined();
		expect(sharedExports.SearchBar).toBeDefined();
		expect(sharedExports.EmptyState).toBeDefined();
		expect(sharedExports.LoadingSpinner).toBeDefined();
		expect(sharedExports.FavoriteButton).toBeDefined();
		expect(sharedExports.Badge).toBeDefined();
		expect(sharedExports.ActionMenu).toBeDefined();
		expect(sharedExports.ActionMenuItem).toBeDefined();
	});
});
