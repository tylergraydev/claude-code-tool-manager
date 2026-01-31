import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { Mcp } from '$lib/types';

describe('DragDrop Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	const createMockMcp = (overrides: Partial<Mcp> = {}): Mcp => ({
		id: 1,
		name: 'test-mcp',
		type: 'stdio',
		command: '/usr/bin/test',
		args: [],
		env: {},
		source: 'user',
		createdAt: '2024-01-01T00:00:00Z',
		updatedAt: '2024-01-01T00:00:00Z',
		...overrides
	});

	describe('initial state', () => {
		it('should have correct initial values', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');

			expect(dragDrop.isDragging).toBe(false);
			expect(dragDrop.draggedMcp).toBeNull();
			expect(dragDrop.dropTarget).toBeNull();
		});
	});

	describe('startDrag', () => {
		it('should set isDragging to true and store the MCP', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp({ id: 1, name: 'test-mcp' });

			dragDrop.startDrag(mcp);

			expect(dragDrop.isDragging).toBe(true);
			expect(dragDrop.draggedMcp).toEqual(mcp);
		});

		it('should replace previous dragged MCP', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp1 = createMockMcp({ id: 1, name: 'mcp-1' });
			const mcp2 = createMockMcp({ id: 2, name: 'mcp-2' });

			dragDrop.startDrag(mcp1);
			dragDrop.startDrag(mcp2);

			expect(dragDrop.draggedMcp).toEqual(mcp2);
		});
	});

	describe('setDropTarget', () => {
		it('should set drop target to project', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');

			dragDrop.setDropTarget({ type: 'project', projectId: 123 });

			expect(dragDrop.dropTarget).toEqual({ type: 'project', projectId: 123 });
		});

		it('should set drop target to global', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');

			dragDrop.setDropTarget({ type: 'global' });

			expect(dragDrop.dropTarget).toEqual({ type: 'global' });
		});

		it('should set drop target to null', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');

			dragDrop.setDropTarget({ type: 'global' });
			dragDrop.setDropTarget(null);

			expect(dragDrop.dropTarget).toBeNull();
		});
	});

	describe('endDrag', () => {
		it('should reset all drag state', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp();

			dragDrop.startDrag(mcp);
			dragDrop.setDropTarget({ type: 'global' });

			expect(dragDrop.isDragging).toBe(true);
			expect(dragDrop.draggedMcp).not.toBeNull();
			expect(dragDrop.dropTarget).not.toBeNull();

			dragDrop.endDrag();

			expect(dragDrop.isDragging).toBe(false);
			expect(dragDrop.draggedMcp).toBeNull();
			expect(dragDrop.dropTarget).toBeNull();
		});
	});

	describe('getDragData', () => {
		it('should return JSON stringified MCP when dragging', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp({ id: 1, name: 'test-mcp' });

			dragDrop.startDrag(mcp);
			const data = dragDrop.getDragData();

			expect(data).toBe(JSON.stringify(mcp));
			expect(JSON.parse(data)).toEqual(mcp);
		});

		it('should return empty string when not dragging', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');

			const data = dragDrop.getDragData();

			expect(data).toBe('');
		});

		it('should return empty string after endDrag', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp();

			dragDrop.startDrag(mcp);
			dragDrop.endDrag();
			const data = dragDrop.getDragData();

			expect(data).toBe('');
		});
	});

	describe('canDrop', () => {
		it('should be false when not dragging', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');

			dragDrop.setDropTarget({ type: 'global' });

			expect(dragDrop.canDrop).toBe(false);
		});

		it('should be false when dragging but no drop target', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp();

			dragDrop.startDrag(mcp);

			expect(dragDrop.canDrop).toBe(false);
		});

		it('should be true when dragging and drop target is set to project', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp();

			dragDrop.startDrag(mcp);
			dragDrop.setDropTarget({ type: 'project', projectId: 1 });

			expect(dragDrop.canDrop).toBe(true);
		});

		it('should be true when dragging and drop target is set to global', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp();

			dragDrop.startDrag(mcp);
			dragDrop.setDropTarget({ type: 'global' });

			expect(dragDrop.canDrop).toBe(true);
		});

		it('should become false after endDrag', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp();

			dragDrop.startDrag(mcp);
			dragDrop.setDropTarget({ type: 'global' });
			expect(dragDrop.canDrop).toBe(true);

			dragDrop.endDrag();
			expect(dragDrop.canDrop).toBe(false);
		});

		it('should become false when drop target is cleared', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp();

			dragDrop.startDrag(mcp);
			dragDrop.setDropTarget({ type: 'global' });
			expect(dragDrop.canDrop).toBe(true);

			dragDrop.setDropTarget(null);
			expect(dragDrop.canDrop).toBe(false);
		});
	});
});
