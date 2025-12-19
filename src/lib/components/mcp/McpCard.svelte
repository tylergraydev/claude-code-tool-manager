<script lang="ts">
	import type { Mcp } from '$lib/types';
	import { dragDrop } from '$lib/stores';
	import { Plug, Globe, Server, GripVertical, MoreVertical, Edit, Copy, Trash2 } from 'lucide-svelte';

	type Props = {
		mcp: Mcp;
		draggable?: boolean;
		showActions?: boolean;
		onEdit?: (mcp: Mcp) => void;
		onDelete?: (mcp: Mcp) => void;
		onDuplicate?: (mcp: Mcp) => void;
		onToggle?: (mcp: Mcp, enabled: boolean) => void;
	};

	let {
		mcp,
		draggable = true,
		showActions = true,
		onEdit,
		onDelete,
		onDuplicate,
		onToggle
	}: Props = $props();

	let showMenu = $state(false);

	const typeIcons = {
		stdio: Plug,
		sse: Globe,
		http: Server
	};

	const typeColors = {
		stdio: 'bg-purple-100 text-purple-600 dark:bg-purple-900/50 dark:text-purple-400',
		sse: 'bg-green-100 text-green-600 dark:bg-green-900/50 dark:text-green-400',
		http: 'bg-blue-100 text-blue-600 dark:bg-blue-900/50 dark:text-blue-400'
	};

	function handleDragStart(e: DragEvent) {
		if (!draggable) return;
		dragDrop.startDrag(mcp);
		e.dataTransfer?.setData('application/json', JSON.stringify(mcp));
		e.dataTransfer!.effectAllowed = 'copy';
	}

	function handleDragEnd() {
		dragDrop.endDrag();
	}

	function closeMenu() {
		showMenu = false;
	}
</script>

<svelte:window onclick={closeMenu} />

<div
	class="card group relative hover:shadow-md transition-all duration-200"
	class:cursor-grab={draggable}
	class:opacity-50={dragDrop.isDragging && dragDrop.draggedMcp?.id === mcp.id}
	{draggable}
	ondragstart={handleDragStart}
	ondragend={handleDragEnd}
	role={draggable ? 'button' : undefined}
	tabindex={draggable ? 0 : undefined}
>
	<div class="flex items-start gap-3">
		{#if draggable}
			<div class="pt-1 opacity-0 group-hover:opacity-100 transition-opacity cursor-grab">
				<GripVertical class="w-4 h-4 text-gray-400" />
			</div>
		{/if}

		<div class="flex-shrink-0 w-10 h-10 rounded-xl {typeColors[mcp.type]} flex items-center justify-center">
			<svelte:component this={typeIcons[mcp.type]} class="w-5 h-5" />
		</div>

		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2">
				<h3 class="font-medium text-gray-900 dark:text-white truncate">
					{mcp.name}
				</h3>
				{#if mcp.source === 'auto-detected'}
					<span class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-emerald-100 text-emerald-700 dark:bg-emerald-900/50 dark:text-emerald-300">
						Auto
					</span>
				{/if}
			</div>

			{#if mcp.description}
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-2">
					{mcp.description}
				</p>
			{/if}

			<div class="flex items-center gap-2 mt-2 flex-wrap">
				<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300">
					{mcp.type}
				</span>

				{#if mcp.type === 'stdio' && mcp.command}
					<span class="text-xs text-gray-400 dark:text-gray-500 font-mono truncate max-w-[150px]">
						{mcp.command}
					</span>
				{:else if mcp.url}
					<span class="text-xs text-gray-400 dark:text-gray-500 truncate max-w-[150px]">
						{new URL(mcp.url).hostname}
					</span>
				{/if}
			</div>
		</div>

		{#if showActions}
			<div class="flex items-center gap-2">
				{#if onToggle}
					<button
						onclick={() => onToggle?.(mcp, !mcp.isEnabledGlobal)}
						class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2"
						class:bg-primary-600={mcp.isEnabledGlobal}
						class:bg-gray-200={!mcp.isEnabledGlobal}
						class:dark:bg-gray-600={!mcp.isEnabledGlobal}
						role="switch"
						aria-checked={mcp.isEnabledGlobal}
					>
						<span
							class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
							class:translate-x-4={mcp.isEnabledGlobal}
							class:translate-x-0={!mcp.isEnabledGlobal}
						/>
					</button>
				{/if}

				<div class="relative">
					<button
						onclick={(e) => {
							e.stopPropagation();
							showMenu = !showMenu;
						}}
						class="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
					>
						<MoreVertical class="w-4 h-4" />
					</button>

					{#if showMenu}
						<div
							class="absolute right-0 top-full mt-1 w-40 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-10"
							onclick={(e) => e.stopPropagation()}
						>
							{#if onEdit}
								<button
									onclick={() => {
										onEdit(mcp);
										closeMenu();
									}}
									class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
								>
									<Edit class="w-4 h-4" />
									Edit
								</button>
							{/if}
							{#if onDuplicate}
								<button
									onclick={() => {
										onDuplicate(mcp);
										closeMenu();
									}}
									class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
								>
									<Copy class="w-4 h-4" />
									Duplicate
								</button>
							{/if}
							{#if onDelete}
								<button
									onclick={() => {
										onDelete(mcp);
										closeMenu();
									}}
									class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20"
								>
									<Trash2 class="w-4 h-4" />
									Delete
								</button>
							{/if}
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>
