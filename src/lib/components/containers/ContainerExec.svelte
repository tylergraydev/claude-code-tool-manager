<script lang="ts">
	import { containerLibrary } from '$lib/stores';
	import { Send } from 'lucide-svelte';

	type Props = {
		containerId: number;
	};

	let { containerId }: Props = $props();

	type ExecEntry = {
		command: string;
		stdout: string;
		stderr: string;
		exitCode: number;
	};

	let command = $state('');
	let history = $state<ExecEntry[]>([]);
	let isRunning = $state(false);
	let outputEl: HTMLDivElement | undefined = $state();

	async function runCommand() {
		const cmd = command.trim();
		if (!cmd || isRunning) return;

		isRunning = true;
		command = '';

		try {
			// Wrap in sh -c so pipes, redirects, env vars, etc. work
			const result = await containerLibrary.exec(containerId, ['sh', '-c', cmd]);
			history = [...history, {
				command: cmd,
				stdout: result.stdout,
				stderr: result.stderr,
				exitCode: result.exitCode,
			}];
		} catch (e) {
			history = [...history, {
				command: cmd,
				stdout: '',
				stderr: String(e),
				exitCode: -1,
			}];
		} finally {
			isRunning = false;
			requestAnimationFrame(() => {
				if (outputEl) outputEl.scrollTop = outputEl.scrollHeight;
			});
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			runCommand();
		}
	}
</script>

<div class="space-y-3">
	<!-- Output -->
	<div bind:this={outputEl} class="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-200 h-64 overflow-auto" aria-label="Command output">
		{#if history.length === 0}
			<p class="text-gray-500">Run a command to see output here</p>
		{:else}
			{#each history as entry}
				<div class="mb-3">
					<div class="text-green-400">$ {entry.command}</div>
					{#if entry.stdout}
						<pre class="whitespace-pre-wrap text-gray-200 mt-1">{entry.stdout}</pre>
					{/if}
					{#if entry.stderr}
						<pre class="whitespace-pre-wrap text-red-400 mt-1">{entry.stderr}</pre>
					{/if}
					{#if entry.exitCode !== 0}
						<div class="text-yellow-500 text-xs mt-1">exit code: {entry.exitCode}</div>
					{/if}
				</div>
			{/each}
		{/if}
	</div>

	<!-- Input -->
	<div class="flex items-center gap-2">
		<span class="text-sm text-green-500 font-mono shrink-0">$</span>
		<input
			type="text"
			bind:value={command}
			onkeydown={handleKeydown}
			placeholder={isRunning ? 'Running...' : 'Enter command...'}
			disabled={isRunning}
			class="input flex-1 font-mono text-sm"
		/>
		<button onclick={runCommand} disabled={isRunning} class="btn btn-primary p-2" aria-label="Run command">
			<Send class="w-4 h-4" />
		</button>
	</div>
</div>
