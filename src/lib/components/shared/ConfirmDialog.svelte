<script lang="ts">
	import { AlertTriangle } from 'lucide-svelte';

	type Props = {
		open: boolean;
		title: string;
		message: string;
		confirmText?: string;
		cancelText?: string;
		variant?: 'danger' | 'warning' | 'info';
		onConfirm: () => void;
		onCancel: () => void;
	};

	let {
		open,
		title,
		message,
		confirmText = 'Confirm',
		cancelText = 'Cancel',
		variant = 'danger',
		onConfirm,
		onCancel
	}: Props = $props();

	const variants = {
		danger: 'bg-red-600 hover:bg-red-700 focus:ring-red-500',
		warning: 'bg-yellow-600 hover:bg-yellow-700 focus:ring-yellow-500',
		info: 'bg-blue-600 hover:bg-blue-700 focus:ring-blue-500'
	};

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onCancel();
		}
	}
</script>

{#if open}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={onCancel}
		onkeydown={handleKeydown}
		role="dialog"
		aria-modal="true"
		aria-labelledby="dialog-title"
	>
		<div
			class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-md w-full mx-4 p-6"
			onclick={(e) => e.stopPropagation()}
		>
			<div class="flex items-start gap-4">
				<div class="flex-shrink-0 w-10 h-10 rounded-full bg-red-100 dark:bg-red-900/50 flex items-center justify-center">
					<AlertTriangle class="w-5 h-5 text-red-600 dark:text-red-400" />
				</div>
				<div class="flex-1">
					<h3 id="dialog-title" class="text-lg font-semibold text-gray-900 dark:text-white">
						{title}
					</h3>
					<p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
						{message}
					</p>
				</div>
			</div>

			<div class="mt-6 flex justify-end gap-3">
				<button onclick={onCancel} class="btn btn-secondary">
					{cancelText}
				</button>
				<button onclick={onConfirm} class="btn text-white {variants[variant]}">
					{confirmText}
				</button>
			</div>
		</div>
	</div>
{/if}
