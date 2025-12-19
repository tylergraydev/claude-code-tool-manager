import type { Mcp } from '$lib/types';

export type DropTarget =
	| { type: 'project'; projectId: number }
	| { type: 'global' }
	| null;

class DragDropState {
	isDragging = $state(false);
	draggedMcp = $state<Mcp | null>(null);
	dropTarget = $state<DropTarget>(null);

	canDrop = $derived.by(() => {
		return this.isDragging && this.dropTarget !== null;
	});

	startDrag(mcp: Mcp) {
		this.isDragging = true;
		this.draggedMcp = mcp;
	}

	setDropTarget(target: DropTarget) {
		this.dropTarget = target;
	}

	endDrag() {
		this.isDragging = false;
		this.draggedMcp = null;
		this.dropTarget = null;
	}

	getDragData(): string {
		return this.draggedMcp ? JSON.stringify(this.draggedMcp) : '';
	}
}

export const dragDrop = new DragDropState();
