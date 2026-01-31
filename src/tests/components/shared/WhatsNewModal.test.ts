import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import WhatsNewModal from '$lib/components/shared/WhatsNewModal.svelte';

// Create mock whatsNew object
const createMockWhatsNew = () => ({
	isOpen: false,
	isLoading: false,
	release: null as {
		version: string;
		name: string;
		body: string;
		publishedAt: string;
		htmlUrl: string;
	} | null,
	error: null as string | null,
	dismiss: vi.fn()
});

let mockWhatsNew = createMockWhatsNew();

// Mock the whatsNew store
vi.mock('$lib/stores/whatsNew.svelte', () => ({
	get whatsNew() {
		return mockWhatsNew;
	}
}));

// Mock shell plugin
vi.mock('@tauri-apps/plugin-shell', () => ({
	open: vi.fn()
}));

describe('WhatsNewModal', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockWhatsNew = createMockWhatsNew();
	});

	describe('when closed', () => {
		it('should not render anything when isOpen is false', () => {
			mockWhatsNew.isOpen = false;
			const { container } = render(WhatsNewModal);

			expect(container.querySelector('.fixed')).not.toBeInTheDocument();
		});
	});

	describe('when open', () => {
		beforeEach(() => {
			mockWhatsNew.isOpen = true;
		});

		it('should render modal when isOpen is true', () => {
			render(WhatsNewModal);

			expect(screen.getByRole('dialog')).toBeInTheDocument();
		});

		it('should have aria-modal attribute', () => {
			render(WhatsNewModal);

			expect(screen.getByRole('dialog')).toHaveAttribute('aria-modal', 'true');
		});

		it('should have aria-labelledby referencing title', () => {
			render(WhatsNewModal);

			expect(screen.getByRole('dialog')).toHaveAttribute('aria-labelledby', 'whats-new-title');
		});

		it('should render What\'s New title', () => {
			render(WhatsNewModal);

			expect(screen.getByText("What's New")).toBeInTheDocument();
		});

		it('should render close button with aria-label', () => {
			render(WhatsNewModal);

			expect(screen.getByLabelText('Close')).toBeInTheDocument();
		});

		it('should render View on GitHub button', () => {
			render(WhatsNewModal);

			expect(screen.getByText('View on GitHub')).toBeInTheDocument();
		});

		it('should render Got it button', () => {
			render(WhatsNewModal);

			expect(screen.getByText('Got it')).toBeInTheDocument();
		});
	});

	describe('loading state', () => {
		beforeEach(() => {
			mockWhatsNew.isOpen = true;
			mockWhatsNew.isLoading = true;
		});

		it('should show loading spinner when loading', () => {
			const { container } = render(WhatsNewModal);

			const spinner = container.querySelector('.animate-spin');
			expect(spinner).toBeInTheDocument();
		});
	});

	describe('with release data', () => {
		beforeEach(() => {
			mockWhatsNew.isOpen = true;
			mockWhatsNew.release = {
				version: '1.5.0',
				name: 'Version 1.5.0',
				body: '## New Features\n- Added dark mode\n- Performance improvements',
				publishedAt: '2024-01-15T10:00:00Z',
				htmlUrl: 'https://github.com/repo/releases/tag/v1.5.0'
			};
		});

		it('should display version number', () => {
			render(WhatsNewModal);

			expect(screen.getByText('Version 1.5.0')).toBeInTheDocument();
		});

		it('should display formatted publish date', () => {
			render(WhatsNewModal);

			// Date format varies by locale, so just check it contains Released
			expect(screen.getByText(/Released/)).toBeInTheDocument();
		});

		it('should render release body with markdown', () => {
			const { container } = render(WhatsNewModal);

			// The markdown should be rendered as HTML
			const prose = container.querySelector('.prose');
			expect(prose).toBeInTheDocument();
		});
	});

	describe('without release data', () => {
		beforeEach(() => {
			mockWhatsNew.isOpen = true;
			mockWhatsNew.release = null;
			mockWhatsNew.isLoading = false;
		});

		it('should show fallback message when no release', () => {
			render(WhatsNewModal);

			expect(screen.getByText('No release notes available.')).toBeInTheDocument();
		});
	});

	describe('interactions', () => {
		beforeEach(() => {
			mockWhatsNew.isOpen = true;
			mockWhatsNew.release = {
				version: '1.5.0',
				name: 'Version 1.5.0',
				body: 'Test release notes',
				publishedAt: '2024-01-15T10:00:00Z',
				htmlUrl: 'https://github.com/repo/releases/tag/v1.5.0'
			};
		});

		it('should call dismiss when close button clicked', async () => {
			render(WhatsNewModal);

			const closeButton = screen.getByLabelText('Close');
			await fireEvent.click(closeButton);

			expect(mockWhatsNew.dismiss).toHaveBeenCalled();
		});

		it('should call dismiss when Got it button clicked', async () => {
			render(WhatsNewModal);

			const gotItButton = screen.getByText('Got it');
			await fireEvent.click(gotItButton);

			expect(mockWhatsNew.dismiss).toHaveBeenCalled();
		});

		it('should call dismiss when backdrop clicked', async () => {
			render(WhatsNewModal);

			const backdrop = screen.getByRole('dialog');
			await fireEvent.click(backdrop);

			expect(mockWhatsNew.dismiss).toHaveBeenCalled();
		});

		it('should call dismiss on Escape key', async () => {
			render(WhatsNewModal);

			const backdrop = screen.getByRole('dialog');
			await fireEvent.keyDown(backdrop, { key: 'Escape' });

			expect(mockWhatsNew.dismiss).toHaveBeenCalled();
		});

		it('should not dismiss when modal content clicked', async () => {
			render(WhatsNewModal);

			const modalContent = screen.getByRole('document');
			await fireEvent.click(modalContent);

			expect(mockWhatsNew.dismiss).not.toHaveBeenCalled();
		});

		it('should open GitHub link when View on GitHub clicked', async () => {
			const { open } = await import('@tauri-apps/plugin-shell');
			render(WhatsNewModal);

			const githubButton = screen.getByText('View on GitHub');
			await fireEvent.click(githubButton);

			expect(open).toHaveBeenCalledWith('https://github.com/repo/releases/tag/v1.5.0');
		});
	});

	describe('markdown rendering', () => {
		beforeEach(() => {
			mockWhatsNew.isOpen = true;
		});

		it('should render headers', () => {
			mockWhatsNew.release = {
				version: '1.5.0',
				name: 'Test',
				body: '## Heading Two\n### Heading Three',
				publishedAt: '2024-01-15T10:00:00Z',
				htmlUrl: 'https://github.com/repo/releases'
			};

			const { container } = render(WhatsNewModal);

			expect(container.querySelector('h3')).toBeInTheDocument();
			expect(container.querySelector('h4')).toBeInTheDocument();
		});

		it('should render bold text', () => {
			mockWhatsNew.release = {
				version: '1.5.0',
				name: 'Test',
				body: '**bold text**',
				publishedAt: '2024-01-15T10:00:00Z',
				htmlUrl: 'https://github.com/repo/releases'
			};

			const { container } = render(WhatsNewModal);

			expect(container.querySelector('strong')).toBeInTheDocument();
		});

		it('should render bullet points', () => {
			mockWhatsNew.release = {
				version: '1.5.0',
				name: 'Test',
				body: '- Item one\n- Item two',
				publishedAt: '2024-01-15T10:00:00Z',
				htmlUrl: 'https://github.com/repo/releases'
			};

			const { container } = render(WhatsNewModal);

			expect(container.querySelector('ul')).toBeInTheDocument();
			expect(container.querySelectorAll('li').length).toBe(2);
		});

		it('should render inline code', () => {
			mockWhatsNew.release = {
				version: '1.5.0',
				name: 'Test',
				body: 'Use the `npm install` command',
				publishedAt: '2024-01-15T10:00:00Z',
				htmlUrl: 'https://github.com/repo/releases'
			};

			const { container } = render(WhatsNewModal);

			expect(container.querySelector('code')).toBeInTheDocument();
		});

		it('should render links', () => {
			mockWhatsNew.release = {
				version: '1.5.0',
				name: 'Test',
				body: 'Check [the docs](https://example.com)',
				publishedAt: '2024-01-15T10:00:00Z',
				htmlUrl: 'https://github.com/repo/releases'
			};

			const { container } = render(WhatsNewModal);

			const link = container.querySelector('a[href="https://example.com"]');
			expect(link).toBeInTheDocument();
			expect(link).toHaveAttribute('target', '_blank');
			expect(link).toHaveAttribute('rel', 'noopener');
		});
	});

	describe('date formatting', () => {
		beforeEach(() => {
			mockWhatsNew.isOpen = true;
		});

		it('should handle invalid date gracefully', () => {
			mockWhatsNew.release = {
				version: '1.5.0',
				name: 'Test',
				body: 'Test',
				publishedAt: 'invalid-date',
				htmlUrl: 'https://github.com/repo/releases'
			};

			render(WhatsNewModal);

			// Should fall back to original string
			expect(screen.getByText(/Released/)).toBeInTheDocument();
		});
	});

	describe('styling', () => {
		beforeEach(() => {
			mockWhatsNew.isOpen = true;
		});

		it('should have semi-transparent backdrop', () => {
			render(WhatsNewModal);

			const backdrop = screen.getByRole('dialog');
			expect(backdrop).toHaveClass('bg-black/50');
		});

		it('should have max width constraint on modal', () => {
			const { container } = render(WhatsNewModal);

			const modal = container.querySelector('.max-w-lg');
			expect(modal).toBeInTheDocument();
		});

		it('should have max height with overflow scroll', () => {
			const { container } = render(WhatsNewModal);

			const modal = container.querySelector('.max-h-\\[80vh\\]');
			expect(modal).toBeInTheDocument();
		});
	});
});
