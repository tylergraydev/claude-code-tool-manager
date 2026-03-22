<script lang="ts">
	import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from 'lucide-svelte';
	import { notifications, type Notification } from '$lib/stores';

	const icons = {
		success: CheckCircle,
		error: AlertCircle,
		info: Info,
		warning: AlertTriangle
	};

	const colors = {
		success: 'bg-green-50 border-green-200 text-green-800 dark:bg-green-900/50 dark:border-green-800 dark:text-green-200',
		error: 'bg-red-50 border-red-200 text-red-800 dark:bg-red-900/50 dark:border-red-800 dark:text-red-200',
		info: 'bg-blue-50 border-blue-200 text-blue-800 dark:bg-blue-900/50 dark:border-blue-800 dark:text-blue-200',
		warning: 'bg-yellow-50 border-yellow-200 text-yellow-800 dark:bg-yellow-900/50 dark:border-yellow-800 dark:text-yellow-200'
	};

	function handleAction(notification: Notification) {
		notification.action?.onclick();
		notifications.remove(notification.id);
	}
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2" role="status" aria-live="polite" aria-atomic="false">
	{#each notifications.notifications as notification (notification.id)}
		<div
			class="flex items-start gap-3 px-4 py-3 rounded-lg border shadow-lg min-w-[300px] max-w-md animate-slide-in {colors[notification.type]}"
		>
			<svelte:component this={icons[notification.type]} class="w-5 h-5 flex-shrink-0 mt-0.5" aria-hidden="true" />
			<div class="flex-1 min-w-0">
				<p class="text-sm font-medium break-words">{notification.message}</p>
				{#if notification.detail}
					<p class="text-xs mt-1 opacity-80 break-words">{notification.detail}</p>
				{/if}
				{#if notification.action}
					<button
						onclick={() => handleAction(notification)}
						class="text-xs font-medium mt-2 underline underline-offset-2 hover:opacity-70 transition-opacity"
					>
						{notification.action.label}
					</button>
				{/if}
			</div>
			<button
				onclick={() => notifications.remove(notification.id)}
				class="p-1 hover:opacity-70 transition-opacity flex-shrink-0"
				aria-label="Dismiss notification"
			>
				<X class="w-4 h-4" aria-hidden="true" />
			</button>
		</div>
	{/each}
</div>

<style>
	@keyframes slide-in {
		from {
			transform: translateX(100%);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	.animate-slide-in {
		animation: slide-in 0.3s ease-out;
	}
</style>
