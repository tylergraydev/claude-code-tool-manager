import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import Header from '$lib/components/layout/Header.svelte';
import { mcpLibrary, projectsStore } from '$lib/stores';

// Mock the stores
vi.mock('$lib/stores', () => ({
	mcpLibrary: {
		load: vi.fn().mockResolvedValue(undefined)
	},
	projectsStore: {
		loadProjects: vi.fn().mockResolvedValue(undefined),
		loadGlobalMcps: vi.fn().mockResolvedValue(undefined)
	}
}));

describe('Header', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		// Reset document class list
		document.documentElement.classList.remove('dark');
	});

	describe('rendering', () => {
		it('should render title', () => {
			render(Header, { props: { title: 'Dashboard' } });

			expect(screen.getByText('Dashboard')).toBeInTheDocument();
		});

		it('should render title in h2 element', () => {
			render(Header, { props: { title: 'Test Title' } });

			const heading = screen.getByRole('heading', { level: 2 });
			expect(heading).toHaveTextContent('Test Title');
		});

		it('should render subtitle when provided', () => {
			render(Header, { props: { title: 'Main', subtitle: 'Sub description' } });

			expect(screen.getByText('Sub description')).toBeInTheDocument();
		});

		it('should not render subtitle when not provided', () => {
			render(Header, { props: { title: 'Main' } });

			expect(screen.queryByText(/description/i)).not.toBeInTheDocument();
		});

		it('should render refresh button', () => {
			render(Header, { props: { title: 'Test' } });

			const button = screen.getByTitle('Refresh data');
			expect(button).toBeInTheDocument();
		});

		it('should render theme toggle button', () => {
			render(Header, { props: { title: 'Test' } });

			const button = screen.getByTitle('Toggle theme');
			expect(button).toBeInTheDocument();
		});
	});

	describe('refresh functionality', () => {
		it('should call store load functions on refresh click', async () => {
			render(Header, { props: { title: 'Test' } });

			const refreshButton = screen.getByTitle('Refresh data');
			await fireEvent.click(refreshButton);

			expect(mcpLibrary.load).toHaveBeenCalled();
			expect(projectsStore.loadProjects).toHaveBeenCalled();
			expect(projectsStore.loadGlobalMcps).toHaveBeenCalled();
		});

		it('should call all refresh functions in parallel', async () => {
			render(Header, { props: { title: 'Test' } });

			const refreshButton = screen.getByTitle('Refresh data');
			await fireEvent.click(refreshButton);

			// All three should be called
			expect(mcpLibrary.load).toHaveBeenCalledTimes(1);
			expect(projectsStore.loadProjects).toHaveBeenCalledTimes(1);
			expect(projectsStore.loadGlobalMcps).toHaveBeenCalledTimes(1);
		});
	});

	describe('theme toggle functionality', () => {
		it('should toggle dark class on document element', async () => {
			render(Header, { props: { title: 'Test' } });

			const themeButton = screen.getByTitle('Toggle theme');

			// Initial state starts with isDark = true, so first toggle should remove dark
			await fireEvent.click(themeButton);
			expect(document.documentElement.classList.contains('dark')).toBe(false);

			// Toggle back to dark
			await fireEvent.click(themeButton);
			expect(document.documentElement.classList.contains('dark')).toBe(true);
		});
	});

	describe('header element', () => {
		it('should render as header element', () => {
			render(Header, { props: { title: 'Test' } });

			const header = screen.getByRole('banner');
			expect(header).toBeInTheDocument();
		});

		it('should have correct height class', () => {
			render(Header, { props: { title: 'Test' } });

			const header = screen.getByRole('banner');
			expect(header).toHaveClass('h-16');
		});
	});

	describe('button styling', () => {
		it('should have btn and btn-ghost classes on buttons', () => {
			render(Header, { props: { title: 'Test' } });

			const refreshButton = screen.getByTitle('Refresh data');
			const themeButton = screen.getByTitle('Toggle theme');

			expect(refreshButton).toHaveClass('btn', 'btn-ghost');
			expect(themeButton).toHaveClass('btn', 'btn-ghost');
		});
	});
});
