<script lang="ts">
	import type { PermissionScope, AllPermissions } from '$lib/types';
	import { PERMISSION_SCOPE_LABELS } from '$lib/types';
	import { User, FolderOpen, FileText } from 'lucide-svelte';

	type Props = {
		selectedScope: PermissionScope;
		permissions: AllPermissions | null;
		hasProject: boolean;
		onselect: (scope: PermissionScope) => void;
	};

	let { selectedScope, permissions, hasProject, onselect }: Props = $props();

	const scopes: { key: PermissionScope; icon: typeof User }[] = [
		{ key: 'user', icon: User },
		{ key: 'project', icon: FolderOpen },
		{ key: 'local', icon: FileText }
	];

	function getRuleCount(scope: PermissionScope): number {
		if (!permissions) return 0;
		const p =
			scope === 'user'
				? permissions.user
				: scope === 'project'
					? permissions.project
					: permissions.local;
		if (!p) return 0;
		return p.allow.length + p.deny.length + p.ask.length;
	}
</script>

<div class="flex gap-1 bg-gray-100 dark:bg-gray-700/50 rounded-lg p-1">
	{#each scopes as { key, icon }}
		{@const isDisabled = key !== 'user' && !hasProject}
		{@const isActive = selectedScope === key}
		{@const count = getRuleCount(key)}
		<button
			onclick={() => onselect(key)}
			disabled={isDisabled}
			class="flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors flex-1
				{isActive
				? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm'
				: isDisabled
					? 'text-gray-400 dark:text-gray-500 cursor-not-allowed'
					: 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'}"
			title={PERMISSION_SCOPE_LABELS[key].description}
		>
			<svelte:component this={icon} class="w-4 h-4" />
			{PERMISSION_SCOPE_LABELS[key].label}
			{#if count > 0}
				<span
					class="ml-1 px-1.5 py-0.5 text-xs rounded-full
						{isActive
					? 'bg-primary-100 dark:bg-primary-900/50 text-primary-700 dark:text-primary-300'
					: 'bg-gray-200 dark:bg-gray-600 text-gray-600 dark:text-gray-300'}"
				>
					{count}
				</span>
			{/if}
		</button>
	{/each}
</div>
