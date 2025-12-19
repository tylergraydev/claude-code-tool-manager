import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

describe('Notifications Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.useFakeTimers();
		// Mock crypto.randomUUID
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

			notifications.add('info', 'Custom duration', 10000);

			expect(notifications.notifications[0].duration).toBe(10000);
		});

		it('should not auto-dismiss when duration is 0', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.add('error', 'Persistent notification', 0);

			vi.advanceTimersByTime(10000);

			expect(notifications.notifications).toHaveLength(1);
		});

		it('should auto-dismiss after duration', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.add('success', 'Auto dismiss', 3000);

			expect(notifications.notifications).toHaveLength(1);

			vi.advanceTimersByTime(3000);

			expect(notifications.notifications).toHaveLength(0);
		});

		it('should add multiple notifications', async () => {
			let counter = 0;
			vi.stubGlobal('crypto', {
				randomUUID: vi.fn().mockImplementation(() => `uuid-${++counter}`)
			});

			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.add('success', 'First');
			notifications.add('error', 'Second');
			notifications.add('warning', 'Third');

			expect(notifications.notifications).toHaveLength(3);
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

			notifications.add('success', 'First', 0);
			notifications.add('error', 'Second', 0);
			notifications.add('info', 'Third', 0);

			notifications.remove('uuid-2');

			expect(notifications.notifications).toHaveLength(2);
			expect(notifications.notifications.find((n) => n.id === 'uuid-2')).toBeUndefined();
		});

		it('should handle removing non-existent ID gracefully', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.add('success', 'Test', 0);

			// Should not throw
			notifications.remove('non-existent-id');

			expect(notifications.notifications).toHaveLength(1);
		});
	});

	describe('helper methods', () => {
		it('should create success notification', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.success('Operation completed');

			expect(notifications.notifications[0].type).toBe('success');
			expect(notifications.notifications[0].message).toBe('Operation completed');
		});

		it('should create error notification', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.error('Something went wrong');

			expect(notifications.notifications[0].type).toBe('error');
			expect(notifications.notifications[0].message).toBe('Something went wrong');
		});

		it('should create info notification', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.info('For your information');

			expect(notifications.notifications[0].type).toBe('info');
			expect(notifications.notifications[0].message).toBe('For your information');
		});

		it('should create warning notification', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.warning('Be careful');

			expect(notifications.notifications[0].type).toBe('warning');
			expect(notifications.notifications[0].message).toBe('Be careful');
		});

		it('should pass custom duration to helper methods', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.success('Test', 10000);
			notifications.error('Test', 0);

			expect(notifications.notifications[0].duration).toBe(10000);
			expect(notifications.notifications[1].duration).toBe(0);
		});
	});

	describe('clear', () => {
		it('should remove all notifications', async () => {
			let counter = 0;
			vi.stubGlobal('crypto', {
				randomUUID: vi.fn().mockImplementation(() => `uuid-${++counter}`)
			});

			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear(); // Clear any leftover notifications from other tests

			notifications.add('success', 'First', 0);
			notifications.add('error', 'Second', 0);
			notifications.add('warning', 'Third', 0);

			expect(notifications.notifications).toHaveLength(3);

			notifications.clear();

			expect(notifications.notifications).toHaveLength(0);
		});

		it('should work on empty notifications list', async () => {
			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			// Should not throw
			notifications.clear();

			expect(notifications.notifications).toHaveLength(0);
		});
	});

	describe('auto-dismiss behavior', () => {
		it('should dismiss multiple notifications at different times', async () => {
			let counter = 0;
			vi.stubGlobal('crypto', {
				randomUUID: vi.fn().mockImplementation(() => `uuid-${++counter}`)
			});

			const { notifications } = await import('$lib/stores/notifications.svelte');
			notifications.clear();

			notifications.add('success', 'Short', 1000);
			notifications.add('info', 'Medium', 3000);
			notifications.add('warning', 'Long', 5000);

			expect(notifications.notifications).toHaveLength(3);

			vi.advanceTimersByTime(1000);
			expect(notifications.notifications).toHaveLength(2);

			vi.advanceTimersByTime(2000);
			expect(notifications.notifications).toHaveLength(1);

			vi.advanceTimersByTime(2000);
			expect(notifications.notifications).toHaveLength(0);
		});
	});
});
