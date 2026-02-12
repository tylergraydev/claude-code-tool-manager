import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import ConfirmDialog from '$lib/components/shared/ConfirmDialog.svelte';

describe('ConfirmDialog Component', () => {
	const defaultProps = {
		open: true,
		title: 'Delete Item',
		message: 'Are you sure you want to delete this item?',
		onConfirm: vi.fn(),
		onCancel: vi.fn()
	};

	it('should show title and message when open', () => {
		render(ConfirmDialog, { props: defaultProps });

		expect(screen.getByText('Delete Item')).toBeInTheDocument();
		expect(screen.getByText('Are you sure you want to delete this item?')).toBeInTheDocument();
	});

	it('should not render when closed', () => {
		render(ConfirmDialog, { props: { ...defaultProps, open: false } });

		expect(screen.queryByText('Delete Item')).not.toBeInTheDocument();
	});

	it('should call onConfirm when confirm button clicked', async () => {
		const onConfirm = vi.fn();
		render(ConfirmDialog, { props: { ...defaultProps, onConfirm } });

		const confirmButton = screen.getByText('Confirm');
		await fireEvent.click(confirmButton);

		expect(onConfirm).toHaveBeenCalledOnce();
	});

	it('should call onCancel when cancel button clicked', async () => {
		const onCancel = vi.fn();
		render(ConfirmDialog, { props: { ...defaultProps, onCancel } });

		const cancelButton = screen.getByText('Cancel');
		await fireEvent.click(cancelButton);

		expect(onCancel).toHaveBeenCalledOnce();
	});

	it('should display custom button text', () => {
		render(ConfirmDialog, {
			props: { ...defaultProps, confirmText: 'Delete', cancelText: 'Keep' }
		});

		expect(screen.getByText('Delete')).toBeInTheDocument();
		expect(screen.getByText('Keep')).toBeInTheDocument();
	});

	it('should use danger variant styling by default', () => {
		render(ConfirmDialog, { props: defaultProps });

		const confirmButton = screen.getByText('Confirm');
		expect(confirmButton.className).toContain('bg-red-600');
	});

	it('should apply warning variant styling', () => {
		render(ConfirmDialog, { props: { ...defaultProps, variant: 'warning' as const } });

		const confirmButton = screen.getByText('Confirm');
		expect(confirmButton.className).toContain('bg-yellow-600');
	});

	it('should apply info variant styling', () => {
		render(ConfirmDialog, { props: { ...defaultProps, variant: 'info' as const } });

		const confirmButton = screen.getByText('Confirm');
		expect(confirmButton.className).toContain('bg-blue-600');
	});

	it('should have accessible dialog role', () => {
		render(ConfirmDialog, { props: defaultProps });

		const dialog = screen.getByRole('dialog');
		expect(dialog).toBeInTheDocument();
		expect(dialog).toHaveAttribute('aria-modal', 'true');
	});

	it('should call onCancel on Escape key', async () => {
		const onCancel = vi.fn();
		render(ConfirmDialog, { props: { ...defaultProps, onCancel } });

		const dialog = screen.getByRole('dialog');
		await fireEvent.keyDown(dialog, { key: 'Escape' });

		expect(onCancel).toHaveBeenCalledOnce();
	});
});
