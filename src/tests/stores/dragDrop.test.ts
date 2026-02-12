import { describe, it, expect, beforeEach } from 'vitest';
import { vi } from 'vitest';
import type { Mcp } from '$lib/types';
import { createMockMcp, resetIdCounter } from '../factories';

describe('DragDrop Store', () => {
	beforeEach(() => {
		resetIdCounter();
		vi.resetModules();
	});

	it('should start in idle state', async () => {
		const { dragDrop } = await import('$lib/stores/dragDrop.svelte');

		expect(dragDrop.isDragging).toBe(false);
		expect(dragDrop.draggedMcp).toBeNull();
		expect(dragDrop.dropTarget).toBeNull();
	});

	describe('startDrag', () => {
		it('should set isDragging and store draggedMcp', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp({ id: 1, name: 'test-mcp' });

			dragDrop.startDrag(mcp);

			expect(dragDrop.isDragging).toBe(true);
			expect(dragDrop.draggedMcp?.id).toBe(1);
			expect(dragDrop.draggedMcp?.name).toBe('test-mcp');
		});
	});

	describe('setDropTarget', () => {
		it('should update drop target for project', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');

			dragDrop.setDropTarget({ type: 'project', projectId: 5 });

			expect(dragDrop.dropTarget).toEqual({ type: 'project', projectId: 5 });
		});

		it('should update drop target for global', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');

			dragDrop.setDropTarget({ type: 'global' });

			expect(dragDrop.dropTarget).toEqual({ type: 'global' });
		});

		it('should allow setting to null', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');

			dragDrop.setDropTarget({ type: 'global' });
			dragDrop.setDropTarget(null);

			expect(dragDrop.dropTarget).toBeNull();
		});
	});

	describe('canDrop', () => {
		it('should be true when dragging and target set', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp({ id: 1 });

			dragDrop.startDrag(mcp);
			dragDrop.setDropTarget({ type: 'global' });

			expect(dragDrop.canDrop).toBe(true);
		});

		it('should be false when not dragging', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');

			dragDrop.setDropTarget({ type: 'global' });

			expect(dragDrop.canDrop).toBe(false);
		});

		it('should be false when no target set', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp({ id: 1 });

			dragDrop.startDrag(mcp);

			expect(dragDrop.canDrop).toBe(false);
		});
	});

	describe('endDrag', () => {
		it('should reset all state', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp({ id: 1 });

			dragDrop.startDrag(mcp);
			dragDrop.setDropTarget({ type: 'project', projectId: 3 });
			dragDrop.endDrag();

			expect(dragDrop.isDragging).toBe(false);
			expect(dragDrop.draggedMcp).toBeNull();
			expect(dragDrop.dropTarget).toBeNull();
		});
	});

	describe('getDragData', () => {
		it('should return JSON of dragged MCP', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp({ id: 1, name: 'test' });

			dragDrop.startDrag(mcp);
			const data = dragDrop.getDragData();

			expect(data).toBeTruthy();
			const parsed = JSON.parse(data);
			expect(parsed.id).toBe(1);
			expect(parsed.name).toBe('test');
		});

		it('should return empty string when no MCP dragged', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');

			expect(dragDrop.getDragData()).toBe('');
		});
	});

	describe('full drag lifecycle', () => {
		it('should handle complete drag-drop cycle', async () => {
			const { dragDrop } = await import('$lib/stores/dragDrop.svelte');
			const mcp = createMockMcp({ id: 1, name: 'drag-mcp' });

			// Start
			dragDrop.startDrag(mcp);
			expect(dragDrop.isDragging).toBe(true);
			expect(dragDrop.canDrop).toBe(false); // no target yet

			// Over target
			dragDrop.setDropTarget({ type: 'project', projectId: 2 });
			expect(dragDrop.canDrop).toBe(true);

			// Leave target
			dragDrop.setDropTarget(null);
			expect(dragDrop.canDrop).toBe(false);

			// Over new target
			dragDrop.setDropTarget({ type: 'global' });
			expect(dragDrop.canDrop).toBe(true);

			// Drop
			dragDrop.endDrag();
			expect(dragDrop.isDragging).toBe(false);
			expect(dragDrop.draggedMcp).toBeNull();
			expect(dragDrop.dropTarget).toBeNull();
			expect(dragDrop.canDrop).toBe(false);
		});
	});
});
