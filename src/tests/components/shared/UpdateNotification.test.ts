import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import UpdateNotification from '$lib/components/shared/UpdateNotification.svelte';

// Create mock updater object
const createMockUpdater = () => ({
	status: 'idle' as 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'error',
	update: null as { version: string } | null,
	error: null as string | null,
	downloadProgress: 0,
	checkForUpdates: vi.fn(),
	downloadAndInstall: vi.fn(),
	restartApp: vi.fn(),
	dismiss: vi.fn()
});

let mockUpdater = createMockUpdater();

// Mock the updater store
vi.mock('$lib/stores/updater.svelte', () => ({
	get updater() {
		return mockUpdater;
	}
}));

describe('UpdateNotification', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.useFakeTimers();
		mockUpdater = createMockUpdater();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	describe('initial behavior', () => {
		it('should call checkForUpdates after 3 seconds on mount', async () => {
			render(UpdateNotification);

			expect(mockUpdater.checkForUpdates).not.toHaveBeenCalled();

			// Advance timer by 3 seconds
			vi.advanceTimersByTime(3000);

			expect(mockUpdater.checkForUpdates).toHaveBeenCalled();
		});

		it('should not render anything when status is idle', () => {
			mockUpdater.status = 'idle';
			const { container } = render(UpdateNotification);

			// Should have no visible notification elements
			expect(container.querySelector('.fixed')).not.toBeInTheDocument();
		});

		it('should not render anything when status is checking', () => {
			mockUpdater.status = 'checking';
			const { container } = render(UpdateNotification);

			expect(container.querySelector('.fixed')).not.toBeInTheDocument();
		});
	});

	describe('available status', () => {
		beforeEach(() => {
			mockUpdater.status = 'available';
			mockUpdater.update = { version: '2.0.0' };
		});

		it('should render update available notification', () => {
			render(UpdateNotification);

			expect(screen.getByText('Update Available')).toBeInTheDocument();
		});

		it('should display version number', () => {
			render(UpdateNotification);

			expect(screen.getByText(/Version 2\.0\.0 is ready to download/)).toBeInTheDocument();
		});

		it('should render Download button', () => {
			render(UpdateNotification);

			expect(screen.getByText('Download')).toBeInTheDocument();
		});

		it('should render Later button', () => {
			render(UpdateNotification);

			expect(screen.getByText('Later')).toBeInTheDocument();
		});

		it('should call downloadAndInstall when Download clicked', async () => {
			vi.useRealTimers();
			render(UpdateNotification);

			const downloadButton = screen.getByText('Download');
			await fireEvent.click(downloadButton);

			expect(mockUpdater.downloadAndInstall).toHaveBeenCalled();
		});

		it('should call dismiss when Later clicked', async () => {
			vi.useRealTimers();
			render(UpdateNotification);

			const laterButton = screen.getByText('Later');
			await fireEvent.click(laterButton);

			expect(mockUpdater.dismiss).toHaveBeenCalled();
		});

		it('should call dismiss when X button clicked', async () => {
			vi.useRealTimers();
			const { container } = render(UpdateNotification);

			// Find the X button (last button in the notification)
			const xButtons = container.querySelectorAll('button');
			const xButton = xButtons[xButtons.length - 1];
			await fireEvent.click(xButton);

			expect(mockUpdater.dismiss).toHaveBeenCalled();
		});
	});

	describe('downloading status', () => {
		beforeEach(() => {
			mockUpdater.status = 'downloading';
			mockUpdater.downloadProgress = 50;
		});

		it('should render downloading notification', () => {
			render(UpdateNotification);

			expect(screen.getByText('Downloading Update...')).toBeInTheDocument();
		});

		it('should show progress bar', () => {
			const { container } = render(UpdateNotification);

			const progressBar = container.querySelector('[style*="width"]');
			expect(progressBar).toBeInTheDocument();
		});

		it('should reflect download progress in bar width', () => {
			const { container } = render(UpdateNotification);

			const progressBar = container.querySelector('.bg-primary-500.h-2');
			expect(progressBar).toHaveStyle({ width: '50%' });
		});

		it('should cap progress at 100%', () => {
			mockUpdater.downloadProgress = 150;
			const { container } = render(UpdateNotification);

			const progressBar = container.querySelector('.bg-primary-500.h-2');
			expect(progressBar).toHaveStyle({ width: '100%' });
		});
	});

	describe('ready status', () => {
		beforeEach(() => {
			mockUpdater.status = 'ready';
		});

		it('should render ready notification', () => {
			render(UpdateNotification);

			expect(screen.getByText('Update Ready')).toBeInTheDocument();
		});

		it('should show restart message', () => {
			render(UpdateNotification);

			expect(screen.getByText('Restart the app to apply the update.')).toBeInTheDocument();
		});

		it('should render Restart Now button', () => {
			render(UpdateNotification);

			expect(screen.getByText('Restart Now')).toBeInTheDocument();
		});

		it('should call restartApp when Restart Now clicked', async () => {
			vi.useRealTimers();
			render(UpdateNotification);

			const restartButton = screen.getByText('Restart Now');
			await fireEvent.click(restartButton);

			expect(mockUpdater.restartApp).toHaveBeenCalled();
		});

		it('should have green border indicating success', () => {
			const { container } = render(UpdateNotification);

			const notification = container.querySelector('.border-green-200');
			expect(notification).toBeInTheDocument();
		});
	});

	describe('error status', () => {
		beforeEach(() => {
			mockUpdater.status = 'error';
			mockUpdater.error = 'Network connection failed';
		});

		it('should render error notification', () => {
			render(UpdateNotification);

			expect(screen.getByText('Update Error')).toBeInTheDocument();
		});

		it('should display error message', () => {
			render(UpdateNotification);

			expect(screen.getByText('Network connection failed')).toBeInTheDocument();
		});

		it('should render Dismiss button', () => {
			render(UpdateNotification);

			expect(screen.getByText('Dismiss')).toBeInTheDocument();
		});

		it('should call dismiss when Dismiss clicked', async () => {
			vi.useRealTimers();
			render(UpdateNotification);

			const dismissButton = screen.getByText('Dismiss');
			await fireEvent.click(dismissButton);

			expect(mockUpdater.dismiss).toHaveBeenCalled();
		});

		it('should call dismiss when X button clicked', async () => {
			vi.useRealTimers();
			const { container } = render(UpdateNotification);

			const xButtons = container.querySelectorAll('button');
			const xButton = xButtons[xButtons.length - 1];
			await fireEvent.click(xButton);

			expect(mockUpdater.dismiss).toHaveBeenCalled();
		});

		it('should have red border indicating error', () => {
			const { container } = render(UpdateNotification);

			const notification = container.querySelector('.border-red-200');
			expect(notification).toBeInTheDocument();
		});
	});

	describe('positioning', () => {
		it('should be positioned fixed at bottom right', () => {
			mockUpdater.status = 'available';
			mockUpdater.update = { version: '2.0.0' };
			const { container } = render(UpdateNotification);

			const notification = container.querySelector('.fixed');
			expect(notification).toHaveClass('bottom-4', 'right-4', 'z-50');
		});
	});
});
