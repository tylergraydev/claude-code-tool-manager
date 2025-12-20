<script lang="ts">
	import { updater } from '$lib/stores/updater.svelte';
	import { onMount } from 'svelte';
	import { Download, RefreshCw, X, CheckCircle, AlertCircle } from 'lucide-svelte';

	onMount(() => {
		// Check for updates on mount (with a small delay to not block startup)
		setTimeout(() => {
			updater.checkForUpdates();
		}, 3000);
	});

	function handleDownload() {
		updater.downloadAndInstall();
	}

	function handleRestart() {
		updater.restartApp();
	}

	function handleDismiss() {
		updater.dismiss();
	}
</script>

{#if updater.status === 'available' && updater.update}
	<div class="fixed bottom-4 right-4 z-50 max-w-sm bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 p-4">
		<div class="flex items-start gap-3">
			<div class="flex-shrink-0">
				<Download class="w-5 h-5 text-primary-500" />
			</div>
			<div class="flex-1 min-w-0">
				<h3 class="text-sm font-medium text-gray-900 dark:text-white">
					Update Available
				</h3>
				<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
					Version {updater.update.version} is ready to download.
				</p>
				<div class="mt-3 flex gap-2">
					<button
						onclick={handleDownload}
						class="px-3 py-1.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-md transition-colors"
					>
						Download
					</button>
					<button
						onclick={handleDismiss}
						class="px-3 py-1.5 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
					>
						Later
					</button>
				</div>
			</div>
			<button onclick={handleDismiss} class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
				<X class="w-4 h-4" />
			</button>
		</div>
	</div>
{/if}

{#if updater.status === 'downloading'}
	<div class="fixed bottom-4 right-4 z-50 max-w-sm bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 p-4">
		<div class="flex items-start gap-3">
			<div class="flex-shrink-0">
				<RefreshCw class="w-5 h-5 text-primary-500 animate-spin" />
			</div>
			<div class="flex-1 min-w-0">
				<h3 class="text-sm font-medium text-gray-900 dark:text-white">
					Downloading Update...
				</h3>
				<div class="mt-2 w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
					<div
						class="bg-primary-500 h-2 rounded-full transition-all duration-300"
						style="width: {Math.min(updater.downloadProgress, 100)}%"
					></div>
				</div>
			</div>
		</div>
	</div>
{/if}

{#if updater.status === 'ready'}
	<div class="fixed bottom-4 right-4 z-50 max-w-sm bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-green-200 dark:border-green-700 p-4">
		<div class="flex items-start gap-3">
			<div class="flex-shrink-0">
				<CheckCircle class="w-5 h-5 text-green-500" />
			</div>
			<div class="flex-1 min-w-0">
				<h3 class="text-sm font-medium text-gray-900 dark:text-white">
					Update Ready
				</h3>
				<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
					Restart the app to apply the update.
				</p>
				<div class="mt-3">
					<button
						onclick={handleRestart}
						class="px-3 py-1.5 text-sm font-medium text-white bg-green-600 hover:bg-green-700 rounded-md transition-colors"
					>
						Restart Now
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}

{#if updater.status === 'error'}
	<div class="fixed bottom-4 right-4 z-50 max-w-sm bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-red-200 dark:border-red-700 p-4">
		<div class="flex items-start gap-3">
			<div class="flex-shrink-0">
				<AlertCircle class="w-5 h-5 text-red-500" />
			</div>
			<div class="flex-1 min-w-0">
				<h3 class="text-sm font-medium text-gray-900 dark:text-white">
					Update Error
				</h3>
				<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
					{updater.error}
				</p>
				<div class="mt-3">
					<button
						onclick={handleDismiss}
						class="px-3 py-1.5 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
					>
						Dismiss
					</button>
				</div>
			</div>
			<button onclick={handleDismiss} class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
				<X class="w-4 h-4" />
			</button>
		</div>
	</div>
{/if}
