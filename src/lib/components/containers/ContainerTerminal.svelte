<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import type { UnlistenFn } from '@tauri-apps/api/event';

	type Props = {
		containerId: number;
	};

	let { containerId }: Props = $props();

	let terminalEl: HTMLDivElement | undefined = $state();
	let terminal: any = null;
	let fitAddon: any = null;
	let sessionId = crypto.randomUUID();
	let execId = $state<string | null>(null);
	let error = $state<string | null>(null);
	let isConnecting = $state(true);
	let unlistenOutput: UnlistenFn | null = null;
	let unlistenExit: UnlistenFn | null = null;
	let resizeObserver: ResizeObserver | null = null;

	async function initTerminal() {
		if (!terminalEl) return;

		try {
			// Dynamic imports for xterm (avoids SSR issues)
			const { Terminal } = await import('@xterm/xterm');
			const { FitAddon } = await import('@xterm/addon-fit');

			// Import CSS
			await import('@xterm/xterm/css/xterm.css');

			terminal = new Terminal({
				cursorBlink: true,
				fontSize: 13,
				fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', Consolas, monospace",
				theme: {
					background: '#111827',
					foreground: '#e5e7eb',
					cursor: '#3b82f6',
					selectionBackground: '#3b82f650',
					black: '#111827',
					red: '#ef4444',
					green: '#22c55e',
					yellow: '#eab308',
					blue: '#3b82f6',
					magenta: '#a855f7',
					cyan: '#06b6d4',
					white: '#e5e7eb',
				},
				allowProposedApi: true,
			});

			fitAddon = new FitAddon();
			terminal.loadAddon(fitAddon);
			terminal.open(terminalEl);
			fitAddon.fit();

			// Handle user input -> send to backend
			terminal.onData((data: string) => {
				invoke('send_shell_input', { sessionId, data }).catch((e: any) => {
					console.error('Failed to send input:', e);
				});
			});

			// Listen for output from backend
			unlistenOutput = await listen<string>(`terminal-output-${sessionId}`, (event) => {
				terminal?.write(event.payload);
			});

			// Listen for exit
			unlistenExit = await listen(`terminal-exit-${sessionId}`, () => {
				terminal?.write('\r\n\x1b[33mSession ended.\x1b[0m\r\n');
				execId = null;
			});

			// Start the shell session
			const result = await invoke<string>('start_container_shell', {
				id: containerId,
				sessionId,
			});
			execId = result;
			isConnecting = false;

			// Resize observer
			resizeObserver = new ResizeObserver(() => {
				if (fitAddon && terminal) {
					fitAddon.fit();
					if (execId) {
						invoke('resize_shell', {
							execId,
							hostId: 1,
							rows: terminal.rows,
							cols: terminal.cols,
						}).catch(() => {});
					}
				}
			});
			resizeObserver.observe(terminalEl);

		} catch (e) {
			error = String(e);
			isConnecting = false;
		}
	}

	onMount(() => {
		initTerminal();
	});

	onDestroy(() => {
		unlistenOutput?.();
		unlistenExit?.();
		resizeObserver?.disconnect();
		terminal?.dispose();
	});
</script>

<div class="space-y-2">
	{#if error}
		<div class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-400 text-sm" role="alert">
			{error}
		</div>
	{/if}

	{#if isConnecting}
		<div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
			<div class="animate-spin w-4 h-4 border-2 border-primary-500 border-t-transparent rounded-full"></div>
			Connecting to container shell...
		</div>
	{/if}

	<div
		bind:this={terminalEl}
		class="rounded-lg overflow-hidden"
		style="height: 320px; background: #111827;"
	></div>
</div>
