<script lang="ts">
	import { Copy, Check, Terminal, Clock } from 'lucide-svelte';

	interface CliFlag {
		flag: string;
		arg?: string;
		description: string;
		example: string;
	}

	const flags: CliFlag[] = [
		{ flag: '--agent', arg: '<name>', description: 'Run session as a specific subagent', example: 'claude --agent code-reviewer' },
		{ flag: '--baremode', description: 'Minimal startup without plugins or extras', example: 'claude --baremode' },
		{ flag: '--system-prompt', arg: '<text>', description: 'Set a custom system prompt', example: 'claude --system-prompt "You are a Go expert"' },
		{ flag: '--append-system-prompt', arg: '<text>', description: 'Append text to the default system prompt', example: 'claude --append-system-prompt "Always use TypeScript"' },
		{ flag: '--permissions', arg: '<mode>', description: 'Set permission mode (default, allowEdits, bypassPermissions, plan, auto)', example: 'claude --permissions plan' },
		{ flag: '--allowedTools', arg: '<tools>', description: 'Comma-separated list of allowed tools', example: 'claude --allowedTools "Read,Grep,Glob"' },
		{ flag: '--disallowedTools', arg: '<tools>', description: 'Comma-separated list of disallowed tools', example: 'claude --disallowedTools "Bash,Write"' },
		{ flag: '--model', arg: '<id>', description: 'Override the default model', example: 'claude --model claude-opus-4-6' },
		{ flag: '--max-turns', arg: '<n>', description: 'Limit the number of agentic turns', example: 'claude --max-turns 10' },
	];

	// Command builder state
	let selectedFlags = $state<Record<string, boolean>>({});
	let flagValues = $state<Record<string, string>>({});
	let copied = $state(false);

	const builtCommand = $derived.by(() => {
		const parts = ['claude'];
		for (const f of flags) {
			if (selectedFlags[f.flag]) {
				if (f.arg && flagValues[f.flag]) {
					parts.push(`${f.flag} ${flagValues[f.flag]}`);
				} else if (!f.arg) {
					parts.push(f.flag);
				}
			}
		}
		return parts.join(' ');
	});

	const hasSelection = $derived(Object.values(selectedFlags).some(Boolean));

	async function copyCommand() {
		try {
			await navigator.clipboard.writeText(builtCommand);
			copied = true;
			setTimeout(() => (copied = false), 2000);
		} catch {
			// Clipboard API may not be available
		}
	}
</script>

<div class="space-y-6">
	<!-- CLI Flags Reference -->
	<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
		<div class="flex items-center gap-2 mb-1">
			<Terminal class="w-4 h-4 text-gray-500 dark:text-gray-400" />
			<h3 class="text-base font-semibold text-gray-900 dark:text-white">CLI Startup Flags</h3>
		</div>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
			Flags you can pass when launching Claude Code from the command line. Toggle flags below to build a command.
		</p>

		<div class="space-y-2">
			{#each flags as f}
				<label class="flex items-start gap-3 p-2.5 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50 cursor-pointer transition-colors">
					<input
						type="checkbox"
						bind:checked={selectedFlags[f.flag]}
						class="mt-1 rounded border-gray-300 dark:border-gray-600 text-primary-600 focus:ring-primary-500"
					/>
					<div class="flex-1 min-w-0">
						<div class="flex items-baseline gap-2">
							<code class="text-sm font-semibold text-gray-800 dark:text-gray-200">{f.flag}</code>
							{#if f.arg}
								<code class="text-xs text-gray-500 dark:text-gray-400">{f.arg}</code>
							{/if}
						</div>
						<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{f.description}</p>
						{#if selectedFlags[f.flag] && f.arg}
							<input
								type="text"
								bind:value={flagValues[f.flag]}
								placeholder={f.example.split(f.flag + ' ')[1] ?? ''}
								class="input text-xs mt-1.5 w-full max-w-xs font-mono"
							/>
						{/if}
					</div>
				</label>
			{/each}
		</div>

		{#if hasSelection}
			<div class="mt-4 bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
				<div class="flex items-center justify-between mb-1">
					<p class="text-xs font-medium text-gray-500 dark:text-gray-400">Command</p>
					<button
						onclick={copyCommand}
						class="flex items-center gap-1 text-xs text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 transition-colors"
					>
						{#if copied}
							<Check class="w-3 h-3" />
							Copied
						{:else}
							<Copy class="w-3 h-3" />
							Copy
						{/if}
					</button>
				</div>
				<code class="text-sm font-mono text-gray-800 dark:text-gray-200 break-all">{builtCommand}</code>
			</div>
		{/if}
	</div>

	<!-- Scheduling Reference -->
	<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
		<div class="flex items-center gap-2 mb-1">
			<Clock class="w-4 h-4 text-gray-500 dark:text-gray-400" />
			<h3 class="text-base font-semibold text-gray-900 dark:text-white">Scheduling</h3>
		</div>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
			Claude Code supports recurring tasks and scheduled agents via slash commands and tools.
		</p>

		<div class="space-y-3">
			<div class="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
				<code class="text-sm font-semibold text-gray-800 dark:text-gray-200">/loop</code>
				<span class="text-xs text-gray-500 dark:text-gray-400 ml-2">interval command</span>
				<p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
					Run a prompt or slash command on a recurring interval. Defaults to 10 minutes.
				</p>
				<p class="text-xs text-gray-600 dark:text-gray-300 mt-1 font-mono">
					Example: <code>/loop 5m /commit</code>
				</p>
			</div>

			<div class="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
				<code class="text-sm font-semibold text-gray-800 dark:text-gray-200">/schedule</code>
				<p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
					Create persistent scheduled agents that run on a cron schedule. Available in cloud and desktop environments.
				</p>
				<p class="text-xs text-gray-600 dark:text-gray-300 mt-1 font-mono">
					Example: <code>/schedule "daily at 9am" review open PRs</code>
				</p>
			</div>

			<div class="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
				<p class="text-xs font-medium text-gray-600 dark:text-gray-300 mb-1">Available Tools</p>
				<div class="flex flex-wrap gap-2">
					{#each ['CronCreate', 'CronList', 'CronDelete'] as tool}
						<code class="text-xs px-2 py-0.5 rounded bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300">{tool}</code>
					{/each}
				</div>
				<p class="text-xs text-gray-500 dark:text-gray-400 mt-1.5">
					These tools are available within Claude Code for managing scheduled tasks programmatically.
				</p>
			</div>
		</div>
	</div>
</div>
