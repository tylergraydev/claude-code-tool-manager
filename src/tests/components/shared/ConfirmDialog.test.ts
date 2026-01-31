import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import ConfirmDialog from '$lib/components/shared/ConfirmDialog.svelte';

describe('ConfirmDialog', () => {
	const defaultProps = {
		open: true,
		title: 'Test Title',
		message: 'Test message content',
		onConfirm: vi.fn(),
		onCancel: vi.fn()
	};

	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render when open is true', () => {
			render(ConfirmDialog, { props: defaultProps });

			expect(screen.getByRole('dialog')).toBeInTheDocument();
			expect(screen.getByText('Test Title')).toBeInTheDocument();
			expect(screen.getByText('Test message content')).toBeInTheDocument();
		});

		it('should not render when open is false', () => {
			render(ConfirmDialog, { props: { ...defaultProps, open: false } });

			expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
		});

		it('should render with default button texts', () => {
			render(ConfirmDialog, { props: defaultProps });

			expect(screen.getByRole('button', { name: 'Confirm' })).toBeInTheDocument();
			expect(screen.getByRole('button', { name: 'Cancel' })).toBeInTheDocument();
		});

		it('should render with custom button texts', () => {
			render(ConfirmDialog, {
				props: {
					...defaultProps,
					confirmText: 'Delete',
					cancelText: 'Keep'
				}
			});

			expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
			expect(screen.getByRole('button', { name: 'Keep' })).toBeInTheDocument();
		});

		it('should render alert icon', () => {
			const { container } = render(ConfirmDialog, { props: defaultProps });

			const svg = container.querySelector('svg');
			expect(svg).toBeInTheDocument();
		});
	});

	describe('accessibility', () => {
		it('should have aria-modal attribute', () => {
			render(ConfirmDialog, { props: defaultProps });

			const dialog = screen.getByRole('dialog');
			expect(dialog).toHaveAttribute('aria-modal', 'true');
		});

		it('should have aria-labelledby pointing to title', () => {
			render(ConfirmDialog, { props: defaultProps });

			const dialog = screen.getByRole('dialog');
			expect(dialog).toHaveAttribute('aria-labelledby', 'dialog-title');

			const title = screen.getByText('Test Title');
			expect(title).toHaveAttribute('id', 'dialog-title');
		});
	});

	describe('variants', () => {
		it('should apply danger variant styles by default', () => {
			render(ConfirmDialog, { props: defaultProps });

			const confirmButton = screen.getByRole('button', { name: 'Confirm' });
			expect(confirmButton).toHaveClass('bg-red-600');
		});

		it('should apply warning variant styles', () => {
			render(ConfirmDialog, { props: { ...defaultProps, variant: 'warning' as const } });

			const confirmButton = screen.getByRole('button', { name: 'Confirm' });
			expect(confirmButton).toHaveClass('bg-yellow-600');
		});

		it('should apply info variant styles', () => {
			render(ConfirmDialog, { props: { ...defaultProps, variant: 'info' as const } });

			const confirmButton = screen.getByRole('button', { name: 'Confirm' });
			expect(confirmButton).toHaveClass('bg-blue-600');
		});
	});

	describe('user interactions', () => {
		it('should call onConfirm when confirm button is clicked', async () => {
			const onConfirm = vi.fn();
			render(ConfirmDialog, { props: { ...defaultProps, onConfirm } });

			await fireEvent.click(screen.getByRole('button', { name: 'Confirm' }));

			expect(onConfirm).toHaveBeenCalledTimes(1);
		});

		it('should call onCancel when cancel button is clicked', async () => {
			const onCancel = vi.fn();
			render(ConfirmDialog, { props: { ...defaultProps, onCancel } });

			await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

			expect(onCancel).toHaveBeenCalledTimes(1);
		});

		it('should call onCancel when backdrop is clicked', async () => {
			const onCancel = vi.fn();
			render(ConfirmDialog, { props: { ...defaultProps, onCancel } });

			const backdrop = screen.getByRole('dialog');
			await fireEvent.click(backdrop);

			expect(onCancel).toHaveBeenCalledTimes(1);
		});

		it('should not call onCancel when dialog content is clicked', async () => {
			const onCancel = vi.fn();
			const { container } = render(ConfirmDialog, { props: { ...defaultProps, onCancel } });

			// Click on the dialog content (white box), not the backdrop
			const dialogContent = container.querySelector('.bg-white');
			expect(dialogContent).toBeInTheDocument();
			await fireEvent.click(dialogContent!);

			// onCancel should not be called because of stopPropagation
			expect(onCancel).not.toHaveBeenCalled();
		});

		it('should call onCancel when Escape key is pressed', async () => {
			const onCancel = vi.fn();
			render(ConfirmDialog, { props: { ...defaultProps, onCancel } });

			const dialog = screen.getByRole('dialog');
			await fireEvent.keyDown(dialog, { key: 'Escape' });

			expect(onCancel).toHaveBeenCalledTimes(1);
		});

		it('should not call onCancel when other keys are pressed', async () => {
			const onCancel = vi.fn();
			render(ConfirmDialog, { props: { ...defaultProps, onCancel } });

			const dialog = screen.getByRole('dialog');
			await fireEvent.keyDown(dialog, { key: 'Enter' });
			await fireEvent.keyDown(dialog, { key: 'Tab' });
			await fireEvent.keyDown(dialog, { key: 'a' });

			expect(onCancel).not.toHaveBeenCalled();
		});
	});

	describe('styling', () => {
		it('should have backdrop with correct classes', () => {
			render(ConfirmDialog, { props: defaultProps });

			const dialog = screen.getByRole('dialog');
			expect(dialog).toHaveClass('fixed', 'inset-0', 'z-50');
		});

		it('should have dialog content with correct structure', () => {
			const { container } = render(ConfirmDialog, { props: defaultProps });

			const content = container.querySelector('.bg-white');
			expect(content).toHaveClass('rounded-xl', 'shadow-xl');
		});
	});
});
