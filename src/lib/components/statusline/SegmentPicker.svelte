<script lang="ts">
	import type { SegmentType, StatusLineSegment } from '$lib/types';
	import { SEGMENT_TYPES } from '$lib/types';
	import { Plus, GitBranch, BarChart3, Monitor, SeparatorVertical, WrapText } from 'lucide-svelte';

	type Props = {
		onAdd: (segment: StatusLineSegment) => void;
	};

	let { onAdd }: Props = $props();

	let openGroup = $state<string | null>(null);

	interface SegmentGroup {
		key: string;
		label: string;
		icon: typeof Plus;
		types: SegmentType[];
	}

	const groups: SegmentGroup[] = [
		{
			key: 'model',
			label: 'Model & Session',
			icon: Monitor,
			types: ['model', 'version', 'session_id', 'agent_name', 'vim_mode']
		},
		{
			key: 'usage',
			label: 'Usage & Cost',
			icon: BarChart3,
			types: ['cost', 'context', 'context_remaining', 'tokens_in', 'tokens_out', 'five_hour_usage', 'weekly_usage', 'duration', 'api_duration', 'lines_changed']
		},
		{
			key: 'git',
			label: 'Git & Workspace',
			icon: GitBranch,
			types: ['git_branch', 'git_status', 'cwd', 'project_dir']
		},
		{
			key: 'other',
			label: 'Custom',
			icon: Plus,
			types: ['custom_text']
		}
	];

	function addSegment(type: SegmentType) {
		const meta = SEGMENT_TYPES.find((t) => t.type === type);
		const segment: StatusLineSegment = {
			id: crypto.randomUUID(),
			type,
			enabled: true,
			color: meta?.defaultColor || 'white',
			format: meta?.formats?.[0]?.value,
			position: Date.now(),
			separatorChar: type === 'separator' ? '|' : undefined,
			customText: type === 'custom_text' ? '' : undefined
		};
		onAdd(segment);
		openGroup = null;
	}

	function addSeparator() {
		addSegment('separator');
	}

	function toggleGroup(key: string) {
		openGroup = openGroup === key ? null : key;
	}

	function getSegmentMeta(type: SegmentType) {
		return SEGMENT_TYPES.find((t) => t.type === type);
	}
</script>

<div class="flex items-center gap-1.5">
	{#each groups as group}
		<div class="relative">
			<button
				onclick={() => toggleGroup(group.key)}
				class="btn btn-secondary text-xs px-2.5 py-1.5 {openGroup === group.key ? 'ring-2 ring-primary-400' : ''}"
			>
				<svelte:component this={group.icon} class="w-3.5 h-3.5 mr-1" />
				{group.label}
			</button>

			{#if openGroup === group.key}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div class="fixed inset-0 z-40" onclick={() => (openGroup = null)}></div>
				<div class="absolute left-0 mt-2 w-64 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-50 max-h-72 overflow-auto">
					{#each group.types as type}
						{@const meta = getSegmentMeta(type)}
						{#if meta}
							<button
								onclick={() => addSegment(type)}
								class="w-full flex flex-col px-3 py-2 text-left hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
							>
								<span class="text-sm font-medium text-gray-700 dark:text-gray-300">{meta.label}</span>
								<span class="text-xs text-gray-400 dark:text-gray-500">{meta.description}</span>
							</button>
						{/if}
					{/each}
				</div>
			{/if}
		</div>
	{/each}

	<button
		onclick={addSeparator}
		class="btn btn-secondary text-xs px-2.5 py-1.5"
		title="Add separator ( | )"
	>
		<SeparatorVertical class="w-3.5 h-3.5 mr-1" />
		Separator
	</button>

	<button
		onclick={() => addSegment('line_break')}
		class="btn btn-secondary text-xs px-2.5 py-1.5"
		title="Add line break (new line)"
	>
		<WrapText class="w-3.5 h-3.5 mr-1" />
		Line Break
	</button>
</div>
