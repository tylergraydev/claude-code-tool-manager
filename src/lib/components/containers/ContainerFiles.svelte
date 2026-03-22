<script lang="ts">
	import { containerLibrary } from '$lib/stores';
	import { Folder, File, ArrowLeft, RefreshCw } from 'lucide-svelte';

	type Props = {
		containerId: number;
		workingDir?: string;
	};

	let { containerId, workingDir }: Props = $props();

	type FileEntry = {
		name: string;
		isDir: boolean;
		size: string;
		modified: string;
	};

	let currentPath = $state(workingDir || '/');
	let entries = $state<FileEntry[]>([]);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	async function listDir(path: string) {
		isLoading = true;
		error = null;
		try {
			const result = await containerLibrary.exec(containerId, ['ls', '-la', '--time-style=long-iso', path]);
			if (result.exitCode !== 0) {
				error = result.stderr || 'Failed to list directory';
				entries = [];
				return;
			}

			const lines = result.stdout.split('\n').filter(l => l.trim());
			const parsed: FileEntry[] = [];

			for (const line of lines) {
				if (line.startsWith('total')) continue;
				// Parse ls -la output: permissions links owner group size date time name
				const parts = line.split(/\s+/);
				if (parts.length < 8) continue;
				const name = parts.slice(7).join(' ');
				if (name === '.' || name === '..') continue;

				parsed.push({
					name,
					isDir: line.startsWith('d'),
					size: parts[4],
					modified: `${parts[5]} ${parts[6]}`,
				});
			}

			// Sort: directories first, then alphabetical
			parsed.sort((a, b) => {
				if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
				return a.name.localeCompare(b.name);
			});

			entries = parsed;
			currentPath = path;
		} catch (e) {
			error = String(e);
			entries = [];
		} finally {
			isLoading = false;
		}
	}

	function navigateTo(name: string) {
		const newPath = currentPath === '/' ? `/${name}` : `${currentPath}/${name}`;
		listDir(newPath);
	}

	function goUp() {
		const parent = currentPath.split('/').slice(0, -1).join('/') || '/';
		listDir(parent);
	}

	function formatSize(size: string): string {
		const n = parseInt(size);
		if (isNaN(n)) return size;
		if (n < 1024) return `${n} B`;
		if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
		return `${(n / (1024 * 1024)).toFixed(1)} MB`;
	}

	$effect(() => {
		const _id = containerId;
		listDir(currentPath);
	});
</script>

<div class="space-y-3">
	<!-- Path bar -->
	<div class="flex items-center gap-2">
		<button onclick={goUp} disabled={currentPath === '/'} class="btn btn-ghost p-1.5" aria-label="Go up">
			<ArrowLeft class="w-4 h-4" />
		</button>
		<div class="flex-1 min-w-0">
			<p class="text-sm font-mono text-gray-700 dark:text-gray-300 truncate">{currentPath}</p>
		</div>
		<button onclick={() => listDir(currentPath)} class="btn btn-ghost p-1.5" aria-label="Refresh" disabled={isLoading}>
			<RefreshCw class="w-4 h-4" />
		</button>
	</div>

	{#if error}
		<div class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-400 text-sm" role="alert">
			{error}
		</div>
	{/if}

	{#if isLoading}
		<div class="flex items-center justify-center py-8" role="status" aria-label="Loading files">
			<div class="animate-spin w-6 h-6 border-2 border-primary-500 border-t-transparent rounded-full"></div>
		</div>
	{:else if entries.length === 0 && !error}
		<p class="text-sm text-gray-500 dark:text-gray-400 text-center py-4">Directory is empty</p>
	{:else}
		<div class="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
			<table class="w-full text-sm">
				<thead>
					<tr class="bg-gray-50 dark:bg-gray-800 text-left text-xs text-gray-500 dark:text-gray-400">
						<th class="px-3 py-2 font-medium">Name</th>
						<th class="px-3 py-2 font-medium w-24 text-right">Size</th>
						<th class="px-3 py-2 font-medium w-40 text-right hidden sm:table-cell">Modified</th>
					</tr>
				</thead>
				<tbody>
					{#each entries as entry}
						<tr class="border-t border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800/50">
							<td class="px-3 py-1.5">
								{#if entry.isDir}
									<button onclick={() => navigateTo(entry.name)} class="flex items-center gap-2 text-primary-600 dark:text-primary-400 hover:underline">
										<Folder class="w-4 h-4 shrink-0" />
										<span class="truncate">{entry.name}</span>
									</button>
								{:else}
									<div class="flex items-center gap-2 text-gray-700 dark:text-gray-300">
										<File class="w-4 h-4 shrink-0 text-gray-400" />
										<span class="truncate">{entry.name}</span>
									</div>
								{/if}
							</td>
							<td class="px-3 py-1.5 text-right text-gray-500 dark:text-gray-400 font-mono text-xs">
								{entry.isDir ? '—' : formatSize(entry.size)}
							</td>
							<td class="px-3 py-1.5 text-right text-gray-400 dark:text-gray-500 text-xs hidden sm:table-cell">
								{entry.modified}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
