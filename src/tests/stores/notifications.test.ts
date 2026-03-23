import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

describe('Notifications Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.useFakeTimers();
		vi.stubGlobal('crypto', {
			randomUUID: vi.fn().mockReturnValue('test-uuid-1234')
		});
	});

	afterEach(() => {
		vi.useRealTimers();
		vi.unstubAllGlobals();
	});

	describe('add', () => {
		it('should add a notification with correct properties', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			const id = notifications.add('success', 'Test message');

			expect(id).toBe('test-uuid-1234');
			expect(notifications.notifications).toHaveLength(1);
			expect(notifications.notifications[0]).toEqual({
				id: 'test-uuid-1234',
				type: 'success',
				message: 'Test message',
				duration: 5000
			});
		});

		it('should use custom duration', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.add('info', 'Custom duration', { duration: 10000 });

			expect(notifications.notifications[0].duration).toBe(10000);
		});
	});

	describe('remove', () => {
		it('should remove notification by ID', async () => {
			let counter = 0;
			vi.stubGlobal('crypto', {
				randomUUID: vi.fn().mockImplementation(() => `uuid-${++counter}`)
			});

			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.add('success', 'First');
			notifications.add('error', 'Second');
			notifications.add('info', 'Third');

			expect(notifications.notifications).toHaveLength(3);

			notifications.remove('uuid-2');

			expect(notifications.notifications).toHaveLength(2);
			expect(notifications.notifications[0].id).toBe('uuid-1');
			expect(notifications.notifications[1].id).toBe('uuid-3');
		});
	});

	describe('clear', () => {
		it('should clear all notifications', async () => {
			let counter = 0;
			vi.stubGlobal('crypto', {
				randomUUID: vi.fn().mockImplementation(() => `uuid-${++counter}`)
			});

			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.add('success', 'First');
			notifications.add('error', 'Second');

			expect(notifications.notifications).toHaveLength(2);

			notifications.clear();

			expect(notifications.notifications).toHaveLength(0);
		});
	});

	describe('helper methods', () => {
		it('should call success method', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.success('Success message');

			expect(notifications.notifications).toHaveLength(1);
			expect(notifications.notifications[0].type).toBe('success');
			expect(notifications.notifications[0].message).toBe('Success message');
		});

		it('should call error method', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.error('Error message');

			expect(notifications.notifications).toHaveLength(1);
			expect(notifications.notifications[0].type).toBe('error');
			expect(notifications.notifications[0].message).toBe('Error message');
		});

		it('should call info method', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.info('Info message');

			expect(notifications.notifications).toHaveLength(1);
			expect(notifications.notifications[0].type).toBe('info');
			expect(notifications.notifications[0].message).toBe('Info message');
		});

		it('should call warning method', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.warning('Warning message');

			expect(notifications.notifications).toHaveLength(1);
			expect(notifications.notifications[0].type).toBe('warning');
			expect(notifications.notifications[0].message).toBe('Warning message');
		});
	});

	describe('auto-dismiss', () => {
		it('should not auto-dismiss when duration is 0', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.add('error', 'Persistent notification', { duration: 0 });

			vi.advanceTimersByTime(10000);

			expect(notifications.notifications).toHaveLength(1);
		});

		it('should auto-dismiss after duration', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.add('success', 'Auto dismiss', { duration: 3000 });

			expect(notifications.notifications).toHaveLength(1);

			vi.advanceTimersByTime(3000);

			expect(notifications.notifications).toHaveLength(0);
		});
	});
});
