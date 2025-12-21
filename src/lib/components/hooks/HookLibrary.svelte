<script lang="ts">
	import type { Hook, HookEventType, GlobalHook, ProjectHook } from '$lib/types';
	import { HOOK_EVENT_TYPES } from '$lib/types';
	import { hookLibrary } from '$lib/stores';
	import HookCard from './HookCard.svelte';
	import { SearchBar } from '$lib/components/shared';
	import { Zap, List, FolderTree, Globe, FolderOpen, Inbox } from 'lucide-svelte';

	type Props = {
		onEdit?: (hook: Hook) => void;
		onDelete?: (hook: Hook) => void;
		onDuplicate?: (hook: Hook) => void;
	};

	let { onEdit, onDelete, onDuplicate }: Props = $props();

	// Event order for consistent grouping
	const EVENT_ORDER = [
		'SessionStart',
		'UserPromptSubmit',
		'PreToolUse',
		'PostToolUse',
		'Notification',
		'Stop',
		'SubagentStop',
		'SessionEnd'
	];

	// Event type styling
	const eventTypeStyles: Record<string, { bg: string; text: string; badgeBg: string; badgeText: string; label: string }> = {
		SessionStart: {
			bg: 'bg-green-50 dark:bg-green-900/20',
			text: 'text-green-600 dark:text-green-400',
			badgeBg: 'bg-green-100 dark:bg-green-900/50',
			badgeText: 'text-green-700 dark:text-green-300',
			label: 'Session Start'
		},
		UserPromptSubmit: {
			bg: 'bg-blue-50 dark:bg-blue-900/20',
			text: 'text-blue-600 dark:text-blue-400',
			badgeBg: 'bg-blue-100 dark:bg-blue-900/50',
			badgeText: 'text-blue-700 dark:text-blue-300',
			label: 'User Prompt Submit'
		},
		PreToolUse: {
			bg: 'bg-amber-50 dark:bg-amber-900/20',
			text: 'text-amber-600 dark:text-amber-400',
			badgeBg: 'bg-amber-100 dark:bg-amber-900/50',
			badgeText: 'text-amber-700 dark:text-amber-300',
			label: 'Pre Tool Use'
		},
		PostToolUse: {
			bg: 'bg-purple-50 dark:bg-purple-900/20',
			text: 'text-purple-600 dark:text-purple-400',
			badgeBg: 'bg-purple-100 dark:bg-purple-900/50',
			badgeText: 'text-purple-700 dark:text-purple-300',
			label: 'Post Tool Use'
		},
		Notification: {
			bg: 'bg-cyan-50 dark:bg-cyan-900/20',
			text: 'text-cyan-600 dark:text-cyan-400',
			badgeBg: 'bg-cyan-100 dark:bg-cyan-900/50',
			badgeText: 'text-cyan-700 dark:text-cyan-300',
			label: 'Notification'
		},
		Stop: {
			bg: 'bg-red-50 dark:bg-red-900/20',
			text: 'text-red-600 dark:text-red-400',
			badgeBg: 'bg-red-100 dark:bg-red-900/50',
			badgeText: 'text-red-700 dark:text-red-300',
			label: 'Stop'
		},
		SubagentStop: {
			bg: 'bg-orange-50 dark:bg-orange-900/20',
			text: 'text-orange-600 dark:text-orange-400',
			badgeBg: 'bg-orange-100 dark:bg-orange-900/50',
			badgeText: 'text-orange-700 dark:text-orange-300',
			label: 'Subagent Stop'
		},
		SessionEnd: {
			bg: 'bg-gray-50 dark:bg-gray-800/50',
			text: 'text-gray-600 dark:text-gray-400',
			badgeBg: 'bg-gray-100 dark:bg-gray-700',
			badgeText: 'text-gray-700 dark:text-gray-300',
			label: 'Session End'
		}
	};

	// Helper to group hooks by event type
	function groupByEventType<T extends { hook: Hook } | Hook>(items: T[], getHook: (item: T) => Hook): { eventType: string; items: T[] }[] {
		const groups: Record<string, T[]> = {};
		for (const item of items) {
			const hook = getHook(item);
			if (!groups[hook.eventType]) {
				groups[hook.eventType] = [];
			}
			groups[hook.eventType].push(item);
		}
		return EVENT_ORDER
			.filter((et) => groups[et]?.length > 0)
			.map((eventType) => ({ eventType, items: groups[eventType] }));
	}

	// Filter global hooks based on search/event filter
	const filteredGlobalHooks = $derived.by(() => {
		let result = hookLibrary.globalHooks;

		if (hookLibrary.searchQuery) {
			const query = hookLibrary.searchQuery.toLowerCase();
			result = result.filter(
				(gh) =>
					gh.hook.name.toLowerCase().includes(query) ||
					gh.hook.description?.toLowerCase().includes(query) ||
					gh.hook.tags?.some((t) => t.toLowerCase().includes(query))
			);
		}

		if (hookLibrary.eventFilter) {
			result = result.filter((gh) => gh.hook.eventType === hookLibrary.eventFilter);
		}

		return result;
	});

	// Filter project hooks based on search/event filter
	const filteredProjectsWithHooks = $derived.by(() => {
		return hookLibrary.projectsWithHooks
			.map((pwh) => {
				let hooks = pwh.hooks;

				if (hookLibrary.searchQuery) {
					const query = hookLibrary.searchQuery.toLowerCase();
					hooks = hooks.filter(
						(ph) =>
							ph.hook.name.toLowerCase().includes(query) ||
							ph.hook.description?.toLowerCase().includes(query) ||
							ph.hook.tags?.some((t) => t.toLowerCase().includes(query))
					);
				}

				if (hookLibrary.eventFilter) {
					hooks = hooks.filter((ph) => ph.hook.eventType === hookLibrary.eventFilter);
				}

				return { ...pwh, hooks };
			})
			.filter((pwh) => pwh.hooks.length > 0);
	});

	// Filter unassigned hooks
	const filteredUnassignedHooks = $derived.by(() => {
		let result = hookLibrary.unassignedHooks;

		if (hookLibrary.searchQuery) {
			const query = hookLibrary.searchQuery.toLowerCase();
			result = result.filter(
				(h) =>
					h.name.toLowerCase().includes(query) ||
					h.description?.toLowerCase().includes(query) ||
					h.tags?.some((t) => t.toLowerCase().includes(query))
			);
		}

		if (hookLibrary.eventFilter) {
			result = result.filter((h) => h.eventType === hookLibrary.eventFilter);
		}

		return result;
	});

	// Group global hooks by event type
	const globalHooksByEvent = $derived(groupByEventType(filteredGlobalHooks, (gh) => gh.hook));

	// Group each project's hooks by event type
	const projectsWithHooksByEvent = $derived(
		filteredProjectsWithHooks.map((pwh) => ({
			project: pwh.project,
			hooksByEvent: groupByEventType(pwh.hooks, (ph) => ph.hook)
		}))
	);

	// Group unassigned hooks by event type
	const unassignedHooksByEvent = $derived(groupByEventType(filteredUnassignedHooks, (h) => h));

	const totalScopedCount = $derived(
		filteredGlobalHooks.length +
			filteredProjectsWithHooks.reduce((acc, p) => acc + p.hooks.length, 0) +
			filteredUnassignedHooks.length
	);
</script>

<div class="space-y-4">
	<!-- Filters -->
	<div class="flex flex-wrap items-center gap-4">
		<div class="flex-1 max-w-sm">
			<SearchBar bind:value={hookLibrary.searchQuery} placeholder="Search hooks..." />
		</div>

		<select
			class="input py-1.5 w-40"
			value={hookLibrary.eventFilter}
			onchange={(e) =>
				hookLibrary.setEventFilter((e.target as HTMLSelectElement).value as HookEventType | '')}
		>
			<option value="">All Events</option>
			{#each HOOK_EVENT_TYPES as event}
				<option value={event.value}>{event.label}</option>
			{/each}
		</select>

		<!-- View Mode Toggle -->
		<div class="flex items-center gap-1 bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
			<button
				onclick={() => hookLibrary.setViewMode('all')}
				class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium transition-colors {hookLibrary.viewMode ===
				'all'
					? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm'
					: 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'}"
			>
				<List class="w-4 h-4" />
				All
			</button>
			<button
				onclick={() => hookLibrary.setViewMode('byScope')}
				class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium transition-colors {hookLibrary.viewMode ===
				'byScope'
					? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm'
					: 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'}"
			>
				<FolderTree class="w-4 h-4" />
				By Scope
			</button>
		</div>

		<div class="text-sm text-gray-500 dark:text-gray-400">
			{hookLibrary.filteredHooks.length} hook{hookLibrary.filteredHooks.length !== 1 ? 's' : ''}
		</div>
	</div>

	<!-- Hook Grid -->
	{#if hookLibrary.isLoading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
		</div>
	{:else if hookLibrary.viewMode === 'all'}
		<!-- All View - Grouped by Event Type -->
		{#if hookLibrary.hooksByEventType.length === 0}
			<div class="text-center py-12">
				<Zap class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
				{#if hookLibrary.searchQuery || hookLibrary.eventFilter}
					<h3 class="text-lg font-medium text-gray-900 dark:text-white">No matching hooks</h3>
					<p class="text-gray-500 dark:text-gray-400 mt-1">Try adjusting your filters</p>
				{:else}
					<h3 class="text-lg font-medium text-gray-900 dark:text-white">No hooks in library</h3>
					<p class="text-gray-500 dark:text-gray-400 mt-1">
						Add your first hook to automate Claude Code actions
					</p>
				{/if}
			</div>
		{:else}
			<div class="space-y-8">
				{#each hookLibrary.hooksByEventType as { eventType, hooks } (eventType)}
					{@const style = eventTypeStyles[eventType] || { bg: 'bg-gray-50 dark:bg-gray-800/50', text: 'text-gray-600 dark:text-gray-400', badgeBg: 'bg-gray-100 dark:bg-gray-700', badgeText: 'text-gray-700 dark:text-gray-300', label: eventType }}
					<div>
						<div class="flex items-center gap-2 mb-4">
							<div class="w-8 h-8 rounded-lg {style.badgeBg} flex items-center justify-center">
								<Zap class="w-4 h-4 {style.text}" />
							</div>
							<h3 class="text-lg font-semibold text-gray-900 dark:text-white">{style.label}</h3>
							<span class="px-2 py-0.5 rounded-full text-xs font-medium {style.badgeBg} {style.badgeText}">
								{hooks.length}
							</span>
						</div>
						<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
							{#each hooks as hook (hook.id)}
								<HookCard {hook} {onEdit} {onDelete} {onDuplicate} />
							{/each}
						</div>
					</div>
				{/each}
			</div>
		{/if}
	{:else if hookLibrary.viewMode === 'byScope'}
		<!-- By Scope View - Scope sections with Event Type sub-groups -->
		{#if totalScopedCount === 0}
			<div class="text-center py-12">
				<Zap class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
				{#if hookLibrary.searchQuery || hookLibrary.eventFilter}
					<h3 class="text-lg font-medium text-gray-900 dark:text-white">No matching hooks</h3>
					<p class="text-gray-500 dark:text-gray-400 mt-1">Try adjusting your filters</p>
				{:else}
					<h3 class="text-lg font-medium text-gray-900 dark:text-white">No hooks assigned</h3>
					<p class="text-gray-500 dark:text-gray-400 mt-1">
						Create hooks and assign them to global or project scope
					</p>
				{/if}
			</div>
		{:else}
			<div class="space-y-10">
				<!-- Global Hooks Section -->
				{#if filteredGlobalHooks.length > 0}
					<div>
						<div class="flex items-center gap-2 mb-4">
							<div class="w-8 h-8 rounded-lg bg-blue-100 dark:bg-blue-900/50 flex items-center justify-center">
								<Globe class="w-4 h-4 text-blue-600 dark:text-blue-400" />
							</div>
							<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Global Hooks</h3>
							<span class="px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-300">
								{filteredGlobalHooks.length}
							</span>
						</div>
						<!-- Event type sub-groups -->
						<div class="space-y-6 pl-4 border-l-2 border-blue-200 dark:border-blue-800">
							{#each globalHooksByEvent as { eventType, items } (eventType)}
								{@const style = eventTypeStyles[eventType] || { bg: 'bg-gray-50 dark:bg-gray-800/50', text: 'text-gray-600 dark:text-gray-400', badgeBg: 'bg-gray-100 dark:bg-gray-700', badgeText: 'text-gray-700 dark:text-gray-300', label: eventType }}
								<div>
									<div class="flex items-center gap-2 mb-3">
										<Zap class="w-4 h-4 {style.text}" />
										<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">{style.label}</h4>
										<span class="px-1.5 py-0.5 rounded text-xs {style.badgeBg} {style.badgeText}">
											{items.length}
										</span>
									</div>
									<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
										{#each items as globalHook (globalHook.id)}
											<HookCard hook={globalHook.hook} {onEdit} {onDelete} {onDuplicate} />
										{/each}
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Project Hooks Sections -->
				{#each projectsWithHooksByEvent as { project, hooksByEvent } (project.id)}
					<div>
						<div class="flex items-center gap-2 mb-4">
							<div class="w-8 h-8 rounded-lg bg-amber-100 dark:bg-amber-900/50 flex items-center justify-center">
								<FolderOpen class="w-4 h-4 text-amber-600 dark:text-amber-400" />
							</div>
							<div class="flex-1 min-w-0">
								<h3 class="text-lg font-semibold text-gray-900 dark:text-white truncate">
									{project.name}
								</h3>
								<p class="text-xs text-gray-500 dark:text-gray-400 truncate font-mono">
									{project.path}
								</p>
							</div>
							<span class="px-2 py-0.5 rounded-full text-xs font-medium bg-amber-100 text-amber-700 dark:bg-amber-900/50 dark:text-amber-300">
								{hooksByEvent.reduce((acc, g) => acc + g.items.length, 0)}
							</span>
						</div>
						<!-- Event type sub-groups -->
						<div class="space-y-6 pl-4 border-l-2 border-amber-200 dark:border-amber-800">
							{#each hooksByEvent as { eventType, items } (eventType)}
								{@const style = eventTypeStyles[eventType] || { bg: 'bg-gray-50 dark:bg-gray-800/50', text: 'text-gray-600 dark:text-gray-400', badgeBg: 'bg-gray-100 dark:bg-gray-700', badgeText: 'text-gray-700 dark:text-gray-300', label: eventType }}
								<div>
									<div class="flex items-center gap-2 mb-3">
										<Zap class="w-4 h-4 {style.text}" />
										<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">{style.label}</h4>
										<span class="px-1.5 py-0.5 rounded text-xs {style.badgeBg} {style.badgeText}">
											{items.length}
										</span>
									</div>
									<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
										{#each items as projectHook (projectHook.id)}
											<HookCard hook={projectHook.hook} {onEdit} {onDelete} {onDuplicate} />
										{/each}
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/each}

				<!-- Unassigned Hooks Section -->
				{#if filteredUnassignedHooks.length > 0}
					<div>
						<div class="flex items-center gap-2 mb-4">
							<div class="w-8 h-8 rounded-lg bg-gray-100 dark:bg-gray-700 flex items-center justify-center">
								<Inbox class="w-4 h-4 text-gray-500 dark:text-gray-400" />
							</div>
							<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Unassigned</h3>
							<span class="px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300">
								{filteredUnassignedHooks.length}
							</span>
						</div>
						<!-- Event type sub-groups -->
						<div class="space-y-6 pl-4 border-l-2 border-gray-200 dark:border-gray-700">
							{#each unassignedHooksByEvent as { eventType, items } (eventType)}
								{@const style = eventTypeStyles[eventType] || { bg: 'bg-gray-50 dark:bg-gray-800/50', text: 'text-gray-600 dark:text-gray-400', badgeBg: 'bg-gray-100 dark:bg-gray-700', badgeText: 'text-gray-700 dark:text-gray-300', label: eventType }}
								<div>
									<div class="flex items-center gap-2 mb-3">
										<Zap class="w-4 h-4 {style.text}" />
										<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">{style.label}</h4>
										<span class="px-1.5 py-0.5 rounded text-xs {style.badgeBg} {style.badgeText}">
											{items.length}
										</span>
									</div>
									<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
										{#each items as hook (hook.id)}
											<HookCard {hook} {onEdit} {onDelete} {onDuplicate} />
										{/each}
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		{/if}
	{/if}
</div>
