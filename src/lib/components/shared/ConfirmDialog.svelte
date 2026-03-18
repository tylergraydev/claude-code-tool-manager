<script lang="ts">
	import { AlertTriangle } from 'lucide-svelte';
	import { tick } from 'svelte';

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
		danger: 'bg-red-600 hover:bg-red-700',
		warning: 'bg-yellow-600 hover:bg-yellow-700',
		info: 'bg-blue-600 hover:bg-blue-700'
	};

	let dialogEl = $state<HTMLDivElement>();
	let previouslyFocused: HTMLElement | null = null;

	$effect(() => {
		if (open) {
			previouslyFocused = document.activeElement as HTMLElement;
			tick().then(() => {
				const cancelBtn = dialogEl?.querySelector<HTMLButtonElement>('[data-cancel]');
				cancelBtn?.focus();
			});
		} else if (previouslyFocused) {
			previouslyFocused.focus();
			previouslyFocused = null;
		}
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onCancel();
			return;
		}
		if (e.key === 'Tab') {
			const focusable = dialogEl?.querySelectorAll<HTMLElement>(
				'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
			);
			if (!focusable || focusable.length === 0) return;
			const first = focusable[0];
			const last = focusable[focusable.length - 1];
			if (e.shiftKey && document.activeElement === first) {
				e.preventDefault();
				last.focus();
			} else if (!e.shiftKey && document.activeElement === last) {
				e.preventDefault();
				first.focus();
			}
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
		aria-describedby="dialog-message"
		tabindex="-1"
	>
		<div
			bind:this={dialogEl}
			class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-md w-full mx-4 p-6"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="document"
		>
			<div class="flex items-start gap-4">
				<div class="flex-shrink-0 w-10 h-10 rounded-full bg-red-100 dark:bg-red-900/50 flex items-center justify-center">
					<AlertTriangle class="w-5 h-5 text-red-600 dark:text-red-400" aria-hidden="true" />
				</div>
				<div class="flex-1 min-w-0">
					<h3 id="dialog-title" class="text-lg font-semibold text-gray-900 dark:text-white truncate">
						{title}
					</h3>
					<p id="dialog-message" class="mt-2 text-sm text-gray-500 dark:text-gray-400 break-words">
						{message}
					</p>
				</div>
			</div>

			<div class="mt-6 flex justify-end gap-3">
				<button data-cancel onclick={onCancel} class="btn btn-secondary">
					{cancelText}
				</button>
				<button onclick={onConfirm} class="btn text-white {variants[variant]}">
					{confirmText}
				</button>
			</div>
		</div>
	</div>
{/if}
