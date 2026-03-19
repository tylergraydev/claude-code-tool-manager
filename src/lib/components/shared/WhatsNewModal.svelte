<script lang="ts">
	import { whatsNew } from '$lib/stores/whatsNew.svelte';
	import { X, ExternalLink, Sparkles, Loader2 } from 'lucide-svelte';
	import { open } from '@tauri-apps/plugin-shell';

	function handleDismiss() {
		whatsNew.dismiss();
	}

	async function handleViewOnGitHub() {
		if (whatsNew.release?.htmlUrl) {
			await open(whatsNew.release.htmlUrl);
		}
	}

	function formatDate(dateString: string): string {
		try {
			return new Date(dateString).toLocaleDateString(undefined, {
				year: 'numeric',
				month: 'long',
				day: 'numeric'
			});
		} catch {
			return dateString;
		}
	}

	// Simple markdown to HTML converter for release notes
	function renderMarkdown(text: string): string {
		return text
			// Headers
			.replace(/^### (.+)$/gm, '<h4 class="text-sm font-semibold text-gray-900 dark:text-white mt-4 mb-2">$1</h4>')
			.replace(/^## (.+)$/gm, '<h3 class="text-base font-semibold text-gray-900 dark:text-white mt-4 mb-2">$1</h3>')
			.replace(/^# (.+)$/gm, '<h2 class="text-lg font-bold text-gray-900 dark:text-white mt-4 mb-2">$1</h2>')
			// Bold
			.replace(/\*\*(.+?)\*\*/g, '<strong class="font-semibold">$1</strong>')
			// Italic
			.replace(/\*(.+?)\*/g, '<em>$1</em>')
			// Code blocks
			.replace(/```[\s\S]*?```/g, (match) => {
				const code = match.slice(3, -3).replace(/^\w+\n/, '');
				return `<pre class="bg-gray-100 dark:bg-gray-800 rounded p-2 text-xs overflow-x-auto my-2"><code>${code}</code></pre>`;
			})
			// Inline code
			.replace(/`([^`]+)`/g, '<code class="bg-gray-100 dark:bg-gray-800 px-1 py-0.5 rounded text-sm">$1</code>')
			// Links
			.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" class="text-primary-600 hover:underline" target="_blank" rel="noopener">$1</a>')
			// Bullet points
			.replace(/^- (.+)$/gm, '<li class="ml-4 list-disc">$1</li>')
			// Wrap consecutive list items in ul
			.replace(/(<li[^>]*>.*<\/li>\n?)+/g, '<ul class="my-2 space-y-1">$&</ul>')
			// Line breaks
			.replace(/\n\n/g, '</p><p class="my-2">')
			.replace(/\n/g, '<br>');
	}
</script>

{#if whatsNew.isOpen}
	<!-- Backdrop -->
	<div
		class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4"
		onclick={handleDismiss}
		onkeydown={(e) => e.key === 'Escape' && handleDismiss()}
		role="dialog"
		aria-modal="true"
		aria-labelledby="whats-new-title"
	>
		<!-- Modal -->
		<div
			class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl max-w-lg w-full max-h-[80vh] flex flex-col"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="document"
		>
			<!-- Header -->
			<div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-full bg-gradient-to-br from-primary-500 to-primary-700 flex items-center justify-center">
						<Sparkles class="w-5 h-5 text-white" />
					</div>
					<div>
						<h2 id="whats-new-title" class="text-lg font-semibold text-gray-900 dark:text-white">
							What's New
						</h2>
						{#if whatsNew.release}
							<p class="text-sm text-gray-500 dark:text-gray-400">
								Version {whatsNew.release.version}
							</p>
						{/if}
					</div>
				</div>
				<button
					onclick={handleDismiss}
					class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
					aria-label="Close"
				>
					<X class="w-5 h-5" />
				</button>
			</div>

			<!-- Content -->
			<div class="flex-1 overflow-y-auto p-4">
				{#if whatsNew.isLoading}
					<div class="flex items-center justify-center py-8">
						<div class="w-8 h-8 text-primary-500 animate-spin"><Loader2 class="w-8 h-8" /></div>
					</div>
				{:else if whatsNew.release}
					{#if whatsNew.release.publishedAt}
						<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
							Released {formatDate(whatsNew.release.publishedAt)}
						</p>
					{/if}

					<div class="prose prose-sm dark:prose-invert max-w-none text-gray-700 dark:text-gray-300">
						{@html renderMarkdown(whatsNew.release.body)}
					</div>
				{:else}
					<p class="text-gray-500 dark:text-gray-400">
						No release notes available.
					</p>
				{/if}
			</div>

			<!-- Footer -->
			<div class="flex items-center justify-between p-4 border-t border-gray-200 dark:border-gray-700">
				<button
					onclick={handleViewOnGitHub}
					class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400 hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
				>
					<ExternalLink class="w-4 h-4" />
					View on GitHub
				</button>
				<button
					onclick={handleDismiss}
					class="px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors"
				>
					Got it
				</button>
			</div>
		</div>
	</div>
{/if}
