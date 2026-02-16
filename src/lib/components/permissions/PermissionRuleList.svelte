<script lang="ts">
	import type { PermissionCategory } from '$lib/types';
	import { GripVertical, X, Plus, ShieldAlert, ShieldQuestion, ShieldCheck } from 'lucide-svelte';

	type Props = {
		category: PermissionCategory;
		rules: string[];
		onremove: (index: number) => void;
		onadd: () => void;
		onreorder: (rules: string[]) => void;
	};

	let { category, rules, onremove, onadd, onreorder }: Props = $props();

	let dragIndex = $state<number | null>(null);
	let dropIndex = $state<number | null>(null);

	const categoryConfig = {
		deny: {
			label: 'Deny',
			icon: ShieldAlert,
			borderColor: 'border-red-200 dark:border-red-800/50',
			headerBg: 'bg-red-50 dark:bg-red-900/20',
			headerText: 'text-red-700 dark:text-red-400',
			badgeBg: 'bg-red-100 dark:bg-red-900/40',
			badgeText: 'text-red-600 dark:text-red-400',
			hoverBg: 'hover:bg-red-50 dark:hover:bg-red-900/10',
			btnClass: 'text-red-600 dark:text-red-400 hover:bg-red-100 dark:hover:bg-red-900/30'
		},
		ask: {
			label: 'Ask',
			icon: ShieldQuestion,
			borderColor: 'border-amber-200 dark:border-amber-800/50',
			headerBg: 'bg-amber-50 dark:bg-amber-900/20',
			headerText: 'text-amber-700 dark:text-amber-400',
			badgeBg: 'bg-amber-100 dark:bg-amber-900/40',
			badgeText: 'text-amber-600 dark:text-amber-400',
			hoverBg: 'hover:bg-amber-50 dark:hover:bg-amber-900/10',
			btnClass: 'text-amber-600 dark:text-amber-400 hover:bg-amber-100 dark:hover:bg-amber-900/30'
		},
		allow: {
			label: 'Allow',
			icon: ShieldCheck,
			borderColor: 'border-green-200 dark:border-green-800/50',
			headerBg: 'bg-green-50 dark:bg-green-900/20',
			headerText: 'text-green-700 dark:text-green-400',
			badgeBg: 'bg-green-100 dark:bg-green-900/40',
			badgeText: 'text-green-600 dark:text-green-400',
			hoverBg: 'hover:bg-green-50 dark:hover:bg-green-900/10',
			btnClass: 'text-green-600 dark:text-green-400 hover:bg-green-100 dark:hover:bg-green-900/30'
		}
	};

	const config = $derived(categoryConfig[category]);

	function handleDragStart(index: number) {
		dragIndex = index;
	}

	function handleDragOver(e: DragEvent, index: number) {
		e.preventDefault();
		dropIndex = index;
	}

	function handleDragEnd() {
		if (dragIndex !== null && dropIndex !== null && dragIndex !== dropIndex) {
			const newRules = [...rules];
			const [moved] = newRules.splice(dragIndex, 1);
			newRules.splice(dropIndex, 0, moved);
			onreorder(newRules);
		}
		dragIndex = null;
		dropIndex = null;
	}

	function parseRule(rule: string): { tool: string; specifier: string | null } {
		const match = rule.match(/^([A-Za-z_][A-Za-z0-9_]*)(?:\((.+)\))?$/);
		if (match) {
			return { tool: match[1], specifier: match[2] || null };
		}
		return { tool: rule, specifier: null };
	}
</script>

<div class="border rounded-lg {config.borderColor}">
	<!-- Header -->
	<div class="flex items-center justify-between px-4 py-2.5 {config.headerBg} rounded-t-lg">
		<div class="flex items-center gap-2">
			<svelte:component this={config.icon} class="w-4 h-4 {config.headerText}" />
			<span class="text-sm font-semibold {config.headerText}">{config.label}</span>
			{#if rules.length > 0}
				<span class="px-1.5 py-0.5 text-xs rounded-full {config.badgeBg} {config.badgeText}">
					{rules.length}
				</span>
			{/if}
		</div>
	</div>

	<!-- Rules -->
	<div class="divide-y divide-gray-100 dark:divide-gray-700/50">
		{#each rules as rule, index}
			{@const parsed = parseRule(rule)}
			<div
				class="flex items-center gap-3 px-4 py-2.5 group {config.hoverBg} transition-colors
					{dropIndex === index && dragIndex !== null ? 'bg-gray-100 dark:bg-gray-700/50' : ''}"
				draggable="true"
				ondragstart={() => handleDragStart(index)}
				ondragover={(e) => handleDragOver(e, index)}
				ondragend={handleDragEnd}
				role="listitem"
			>
				<button
					class="cursor-grab text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 opacity-0 group-hover:opacity-100 transition-opacity"
					title="Drag to reorder"
				>
					<GripVertical class="w-4 h-4" />
				</button>
				<div class="flex-1 font-mono text-sm text-gray-800 dark:text-gray-200">
					<span class="font-semibold">{parsed.tool}</span>{#if parsed.specifier}<span
							class="text-gray-500 dark:text-gray-400">({parsed.specifier})</span
						>{/if}
				</div>
				<button
					onclick={() => onremove(index)}
					class="p-1 rounded text-gray-400 hover:text-red-500 dark:hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity"
					title="Remove rule"
				>
					<X class="w-4 h-4" />
				</button>
			</div>
		{/each}

		{#if rules.length === 0}
			<div class="px-4 py-6 text-center text-sm text-gray-400 dark:text-gray-500">
				No {category} rules configured
			</div>
		{/if}
	</div>

	<!-- Add button -->
	<div class="px-4 py-2 border-t border-gray-100 dark:border-gray-700/50">
		<button
			onclick={onadd}
			class="flex items-center gap-2 px-3 py-1.5 text-sm rounded-md transition-colors {config.btnClass}"
		>
			<Plus class="w-4 h-4" />
			Add {config.label} Rule
		</button>
	</div>
</div>
