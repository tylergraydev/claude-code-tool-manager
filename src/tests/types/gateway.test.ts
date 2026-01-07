import { describe, it, expect } from 'vitest';
import type { BackendStatus } from '$lib/types/gateway';

describe('Gateway Types', () => {
	describe('BackendStatus', () => {
		it('should define valid BackendStatus values', () => {
			const validStatuses: BackendStatus[] = ['connecting', 'connected', 'disconnected', 'failed', 'restarting'];

			expect(validStatuses).toHaveLength(5);
		});

		it('should include connecting status', () => {
			const status: BackendStatus = 'connecting';

			expect(status).toBe('connecting');
		});

		it('should include connected status', () => {
			const status: BackendStatus = 'connected';

			expect(status).toBe('connected');
		});

		it('should include disconnected status', () => {
			const status: BackendStatus = 'disconnected';

			expect(status).toBe('disconnected');
		});

		it('should include failed status', () => {
			const status: BackendStatus = 'failed';

			expect(status).toBe('failed');
		});

		it('should include restarting status', () => {
			const status: BackendStatus = 'restarting';

			expect(status).toBe('restarting');
		});
	});
});
