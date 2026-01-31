import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Sidebar from '$lib/components/layout/Sidebar.svelte';

// Mock Tauri app API
vi.mock('@tauri-apps/api/app', () => ({
	getVersion: vi.fn().mockResolvedValue('1.5.0')
}));

describe('Sidebar', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render the app title', () => {
			render(Sidebar);

			expect(screen.getByText('Claude Code')).toBeInTheDocument();
		});

		it('should render the subtitle', () => {
			render(Sidebar);

			expect(screen.getByText('Tool Manager')).toBeInTheDocument();
		});

		it('should render all navigation items', () => {
			render(Sidebar);

			expect(screen.getByText('Dashboard')).toBeInTheDocument();
			expect(screen.getByText('MCP Library')).toBeInTheDocument();
			expect(screen.getByText('Commands')).toBeInTheDocument();
			expect(screen.getByText('Skills')).toBeInTheDocument();
			expect(screen.getByText('Sub-Agents')).toBeInTheDocument();
			expect(screen.getByText('Hooks')).toBeInTheDocument();
			expect(screen.getByText('Marketplace')).toBeInTheDocument();
			expect(screen.getByText('Projects')).toBeInTheDocument();
			expect(screen.getByText('Global Settings')).toBeInTheDocument();
		});

		it('should render navigation links with correct hrefs', () => {
			render(Sidebar);

			expect(screen.getByRole('link', { name: 'Dashboard' })).toHaveAttribute('href', '/');
			expect(screen.getByRole('link', { name: 'MCP Library' })).toHaveAttribute('href', '/library');
			expect(screen.getByRole('link', { name: 'Commands' })).toHaveAttribute('href', '/commands');
			expect(screen.getByRole('link', { name: 'Skills' })).toHaveAttribute('href', '/skills');
			expect(screen.getByRole('link', { name: 'Sub-Agents' })).toHaveAttribute('href', '/subagents');
			expect(screen.getByRole('link', { name: 'Hooks' })).toHaveAttribute('href', '/hooks');
			expect(screen.getByRole('link', { name: 'Marketplace' })).toHaveAttribute('href', '/marketplace');
			expect(screen.getByRole('link', { name: 'Projects' })).toHaveAttribute('href', '/projects');
			expect(screen.getByRole('link', { name: 'Global Settings' })).toHaveAttribute('href', '/settings');
		});
	});

	describe('version display', () => {
		it('should display version after loading', async () => {
			const { getVersion } = await import('@tauri-apps/api/app');
			vi.mocked(getVersion).mockResolvedValueOnce('1.5.0');

			render(Sidebar);

			// Wait for version to load
			await vi.waitFor(() => {
				expect(screen.getByText('v1.5.0')).toBeInTheDocument();
			});
		});

		it('should show fallback version on error', async () => {
			const { getVersion } = await import('@tauri-apps/api/app');
			vi.mocked(getVersion).mockRejectedValueOnce(new Error('Failed'));

			render(Sidebar);

			// Wait for fallback
			await vi.waitFor(() => {
				expect(screen.getByText('v1.0.0')).toBeInTheDocument();
			});
		});
	});

	describe('styling', () => {
		it('should have aside element with correct width', () => {
			const { container } = render(Sidebar);

			const aside = container.querySelector('aside');
			expect(aside).toHaveClass('w-64');
		});

		it('should have navigation element', () => {
			const { container } = render(Sidebar);

			const nav = container.querySelector('nav');
			expect(nav).toBeInTheDocument();
		});
	});
});
