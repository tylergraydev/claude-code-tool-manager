import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import Toast from '$lib/components/shared/Toast.svelte';
import { notifications } from '$lib/stores';

describe('Toast', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		// Clear all notifications before each test
		while (notifications.notifications.length > 0) {
			notifications.remove(notifications.notifications[0].id);
		}
	});

	describe('rendering', () => {
		it('should render empty when no notifications', () => {
			const { container } = render(Toast);

			const toastContainer = container.querySelector('.fixed');
			expect(toastContainer).toBeInTheDocument();
			// Should have no notification items
			expect(screen.queryByRole('button')).not.toBeInTheDocument();
		});

		it('should render success notification', () => {
			notifications.success('Success message');

			render(Toast);

			expect(screen.getByText('Success message')).toBeInTheDocument();
		});

		it('should render error notification', () => {
			notifications.error('Error message');

			render(Toast);

			expect(screen.getByText('Error message')).toBeInTheDocument();
		});

		it('should render info notification', () => {
			notifications.info('Info message');

			render(Toast);

			expect(screen.getByText('Info message')).toBeInTheDocument();
		});

		it('should render warning notification', () => {
			notifications.warning('Warning message');

			render(Toast);

			expect(screen.getByText('Warning message')).toBeInTheDocument();
		});

		it('should render multiple notifications', () => {
			notifications.success('Message 1');
			notifications.error('Message 2');
			notifications.info('Message 3');

			render(Toast);

			expect(screen.getByText('Message 1')).toBeInTheDocument();
			expect(screen.getByText('Message 2')).toBeInTheDocument();
			expect(screen.getByText('Message 3')).toBeInTheDocument();
		});
	});

	describe('dismiss functionality', () => {
		it('should show dismiss button for each notification', () => {
			notifications.success('Test message');

			render(Toast);

			const dismissButton = screen.getByRole('button');
			expect(dismissButton).toBeInTheDocument();
		});

		it('should remove notification when dismiss is clicked', async () => {
			notifications.success('Test message');

			render(Toast);

			expect(screen.getByText('Test message')).toBeInTheDocument();

			const dismissButton = screen.getByRole('button');
			await fireEvent.click(dismissButton);

			expect(screen.queryByText('Test message')).not.toBeInTheDocument();
		});

		it('should only remove clicked notification', async () => {
			notifications.success('Message 1');
			notifications.error('Message 2');

			render(Toast);

			const dismissButtons = screen.getAllByRole('button');
			await fireEvent.click(dismissButtons[0]);

			// First message should be removed, second should remain
			expect(screen.queryByText('Message 1')).not.toBeInTheDocument();
			expect(screen.getByText('Message 2')).toBeInTheDocument();
		});
	});

	describe('styling', () => {
		it('should apply success colors', () => {
			notifications.success('Success');

			const { container } = render(Toast);

			const notification = container.querySelector('.bg-green-50');
			expect(notification).toBeInTheDocument();
		});

		it('should apply error colors', () => {
			notifications.error('Error');

			const { container } = render(Toast);

			const notification = container.querySelector('.bg-red-50');
			expect(notification).toBeInTheDocument();
		});

		it('should apply info colors', () => {
			notifications.info('Info');

			const { container } = render(Toast);

			const notification = container.querySelector('.bg-blue-50');
			expect(notification).toBeInTheDocument();
		});

		it('should apply warning colors', () => {
			notifications.warning('Warning');

			const { container } = render(Toast);

			const notification = container.querySelector('.bg-yellow-50');
			expect(notification).toBeInTheDocument();
		});

		it('should have fixed positioning', () => {
			const { container } = render(Toast);

			const toastContainer = container.querySelector('.fixed');
			expect(toastContainer).toBeInTheDocument();
			expect(toastContainer).toHaveClass('bottom-4', 'right-4', 'z-50');
		});

		it('should have animation class', () => {
			notifications.success('Test');

			const { container } = render(Toast);

			const notification = container.querySelector('.animate-slide-in');
			expect(notification).toBeInTheDocument();
		});
	});

	describe('icons', () => {
		it('should render icon for each notification type', () => {
			notifications.success('Test');

			const { container } = render(Toast);

			// Should have an SVG icon
			const svg = container.querySelector('svg');
			expect(svg).toBeInTheDocument();
		});

		it('should render close icon (X) in dismiss button', () => {
			notifications.success('Test');

			render(Toast);

			const button = screen.getByRole('button');
			const svg = button.querySelector('svg');
			expect(svg).toBeInTheDocument();
		});
	});
});
