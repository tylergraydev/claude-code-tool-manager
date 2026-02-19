<script lang="ts">
	import type { SessionDetail } from '$lib/types';
	import { formatCompactNumber, formatModelName } from '$lib/types/usage';
	import { X, User, Bot, Wrench } from 'lucide-svelte';

	type Props = {
		detail: SessionDetail;
		onClose: () => void;
	};

	let { detail, onClose }: Props = $props();

	function formatTime(iso: string | null): string {
		if (!iso) return '';
		try {
			return new Date(iso).toLocaleTimeString(undefined, {
				hour: '2-digit',
				minute: '2-digit',
				second: '2-digit'
			});
		} catch {
			return iso;
		}
	}

	function roleIcon(role: string) {
		if (role === 'user') return User;
		if (role === 'assistant') return Bot;
		return Wrench;
	}

	function roleBgColor(role: string): string {
		if (role === 'user') return 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800';
		if (role === 'assistant')
			return 'bg-violet-50 dark:bg-violet-900/20 border-violet-200 dark:border-violet-800';
		return 'bg-gray-50 dark:bg-gray-800/50 border-gray-200 dark:border-gray-700';
	}

	function roleBadgeColor(role: string): string {
		if (role === 'user') return 'bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-300';
		if (role === 'assistant')
			return 'bg-violet-100 text-violet-700 dark:bg-violet-900/50 dark:text-violet-300';
		return 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300';
	}

	function roleLabel(role: string): string {
		if (role === 'user') return 'User';
		if (role === 'assistant') return 'Assistant';
		if (role === 'tool_result') return 'Tool Result';
		return role;
	}
</script>

<div
	class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden"
>
	<div
		class="px-4 py-3 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between sticky top-0 bg-white dark:bg-gray-800 z-10"
	>
		<div>
			<h3 class="text-sm font-semibold text-gray-900 dark:text-white">Session Transcript</h3>
			<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
				{detail.messages.length} messages
			</p>
		</div>
		<button
			onclick={onClose}
			class="p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
		>
			<X class="w-4 h-4 text-gray-500" />
		</button>
	</div>

	<div class="p-4 space-y-3 max-h-[600px] overflow-y-auto">
		{#each detail.messages as message}
			<div class="rounded-lg border p-3 {roleBgColor(message.role)}">
				<div class="flex items-center gap-2 mb-1.5">
					<span class="inline-flex items-center gap-1 text-xs font-medium px-2 py-0.5 rounded-full {roleBadgeColor(message.role)}">
						<svelte:component this={roleIcon(message.role)} class="w-3 h-3" />
						{roleLabel(message.role)}
					</span>

					{#if message.model}
						<span
							class="text-xs bg-violet-100 dark:bg-violet-900/30 text-violet-700 dark:text-violet-300 px-1.5 py-0.5 rounded"
						>
							{formatModelName(message.model)}
						</span>
					{/if}

					{#if message.timestamp}
						<span class="text-xs text-gray-400 dark:text-gray-500 ml-auto">
							{formatTime(message.timestamp)}
						</span>
					{/if}
				</div>

				{#if message.contentPreview}
					<p class="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap break-words">
						{message.contentPreview}
					</p>
				{/if}

				{#if message.toolCalls.length > 0}
					<div class="flex flex-wrap gap-1 mt-2">
						{#each message.toolCalls as tool}
							<span
								class="inline-flex items-center gap-1 text-xs bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-300 px-1.5 py-0.5 rounded"
							>
								<Wrench class="w-3 h-3" />
								{tool.toolName}
							</span>
						{/each}
					</div>
				{/if}

				{#if message.usage}
					<div class="flex items-center gap-3 mt-2 text-xs text-gray-400 dark:text-gray-500">
						<span>In: {formatCompactNumber(message.usage.inputTokens)}</span>
						<span>Out: {formatCompactNumber(message.usage.outputTokens)}</span>
						{#if message.usage.cacheReadInputTokens > 0}
							<span>Cache: {formatCompactNumber(message.usage.cacheReadInputTokens)}</span>
						{/if}
					</div>
				{/if}
			</div>
		{/each}
	</div>
</div>
