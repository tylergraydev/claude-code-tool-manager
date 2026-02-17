<script lang="ts">
	import { onMount } from 'svelte';
	import { formatKeystroke, RESERVED_KEYS, TERMINAL_CONFLICT_KEYS } from '$lib/types';
	import type { KeyConflict, KeybindingContext } from '$lib/types';
	import { keybindingsLibrary } from '$lib/stores';
	import { AlertTriangle, X, Keyboard } from 'lucide-svelte';

	interface Props {
		context: KeybindingContext;
		action: string;
		actionLabel: string;
		currentKeys: string[];
		onconfirm: (key: string) => void;
		oncancel: () => void;
	}

	let { context, action, actionLabel, currentKeys, onconfirm, oncancel }: Props = $props();

	let capturedKey = $state('');
	let chordFirst = $state('');
	let chordTimer = $state<ReturnType<typeof setTimeout> | null>(null);
	let isWaitingForChord = $state(false);
	let conflicts = $state<KeyConflict[]>([]);
	let isReserved = $state(false);
	let isTerminalConflict = $state(false);

	const MODIFIER_ORDER = ['alt', 'ctrl', 'meta', 'shift'];

	function normalizeKey(e: KeyboardEvent): string {
		const parts: string[] = [];

		if (e.altKey) parts.push('alt');
		if (e.ctrlKey) parts.push('ctrl');
		if (e.metaKey) parts.push('meta');
		if (e.shiftKey) parts.push('shift');

		let key = e.key.toLowerCase();
		// Normalize special keys
		if (key === ' ') key = 'space';
		else if (key === 'arrowup') key = 'up';
		else if (key === 'arrowdown') key = 'down';
		else if (key === 'arrowleft') key = 'left';
		else if (key === 'arrowright') key = 'right';

		// Skip if only a modifier was pressed
		if (['alt', 'control', 'shift', 'meta'].includes(key)) {
			return '';
		}

		parts.push(key);

		// Sort modifiers in canonical order
		const mods = parts.filter((p) => MODIFIER_ORDER.includes(p));
		mods.sort((a, b) => MODIFIER_ORDER.indexOf(a) - MODIFIER_ORDER.indexOf(b));
		const nonMods = parts.filter((p) => !MODIFIER_ORDER.includes(p));

		return [...mods, ...nonMods].join('+');
	}

	function handleKeyDown(e: KeyboardEvent) {
		e.preventDefault();
		e.stopPropagation();

		const key = normalizeKey(e);
		if (!key) return;

		if (isWaitingForChord) {
			// Second key in chord
			if (chordTimer) clearTimeout(chordTimer);
			chordTimer = null;
			isWaitingForChord = false;

			const chord = `${chordFirst} ${key}`;
			capturedKey = chord;
			checkConflicts(chord);
		} else {
			// First key (or standalone)
			chordFirst = key;
			capturedKey = key;
			checkConflicts(key);

			// Start chord timer
			isWaitingForChord = true;
			if (chordTimer) clearTimeout(chordTimer);
			chordTimer = setTimeout(() => {
				isWaitingForChord = false;
				chordTimer = null;
			}, 1500);
		}
	}

	function checkConflicts(key: string) {
		isReserved = keybindingsLibrary.isReservedKey(key);
		isTerminalConflict = keybindingsLibrary.isTerminalConflict(key);
		conflicts = keybindingsLibrary.detectConflicts(context, key, action);
	}

	function handleConfirm() {
		if (capturedKey && !isReserved) {
			onconfirm(capturedKey);
		}
	}

	function handleClear() {
		capturedKey = '';
		chordFirst = '';
		isWaitingForChord = false;
		if (chordTimer) clearTimeout(chordTimer);
		chordTimer = null;
		conflicts = [];
		isReserved = false;
		isTerminalConflict = false;
	}

	onMount(() => {
		const listener = (e: KeyboardEvent) => handleKeyDown(e);
		window.addEventListener('keydown', listener);
		return () => {
			window.removeEventListener('keydown', listener);
			if (chordTimer) clearTimeout(chordTimer);
		};
	});
</script>

<!-- Backdrop -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
	onmousedown={(e) => { if (e.target === e.currentTarget) oncancel(); }}
>
	<!-- Dialog -->
	<div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-md mx-4 overflow-hidden">
		<!-- Header -->
		<div class="flex items-center justify-between px-5 py-4 border-b border-gray-200 dark:border-gray-700">
			<div class="flex items-center gap-2">
				<Keyboard class="w-5 h-5 text-primary-500" />
				<h3 class="font-semibold text-gray-900 dark:text-white">Capture Keybinding</h3>
			</div>
			<button onclick={oncancel} class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
				<X class="w-5 h-5" />
			</button>
		</div>

		<!-- Body -->
		<div class="p-5 space-y-4">
			<!-- Action info -->
			<div class="text-sm text-gray-500 dark:text-gray-400">
				Setting keybinding for <span class="font-medium text-gray-900 dark:text-white">{actionLabel}</span>
				in <span class="font-medium text-gray-900 dark:text-white">{context}</span> context
			</div>

			<!-- Capture area -->
			<div
				class="relative flex items-center justify-center min-h-[80px] rounded-lg border-2 border-dashed
					{capturedKey
					? 'border-primary-400 bg-primary-50 dark:bg-primary-900/20'
					: 'border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-900/50'}"
			>
				{#if capturedKey}
					<div class="text-center">
						<div class="text-2xl font-mono font-bold text-primary-600 dark:text-primary-400">
							{formatKeystroke(capturedKey)}
						</div>
						{#if isWaitingForChord}
							<div class="text-xs text-gray-500 dark:text-gray-400 mt-1 animate-pulse">
								Press next key for chord, or wait to confirm...
							</div>
						{/if}
					</div>
				{:else}
					<div class="text-center text-gray-400 dark:text-gray-500">
						<p class="text-sm font-medium">Press a key combination...</p>
						<p class="text-xs mt-1">Supports modifier keys and chords</p>
					</div>
				{/if}
			</div>

			<!-- Current keys -->
			{#if currentKeys.length > 0}
				<div class="text-xs text-gray-500 dark:text-gray-400">
					Currently bound:
					{#each currentKeys as key}
						<span class="inline-block px-1.5 py-0.5 mx-0.5 rounded bg-gray-100 dark:bg-gray-700 font-mono text-gray-600 dark:text-gray-300">
							{formatKeystroke(key)}
						</span>
					{/each}
				</div>
			{/if}

			<!-- Warnings -->
			{#if isReserved}
				<div class="flex items-start gap-2 p-3 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800">
					<AlertTriangle class="w-4 h-4 text-red-500 mt-0.5 flex-shrink-0" />
					<div class="text-sm text-red-700 dark:text-red-400">
						<span class="font-medium">{formatKeystroke(capturedKey)}</span> is a reserved key and cannot be rebound.
					</div>
				</div>
			{/if}

			{#if isTerminalConflict && !isReserved}
				<div class="flex items-start gap-2 p-3 rounded-lg bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800">
					<AlertTriangle class="w-4 h-4 text-yellow-500 mt-0.5 flex-shrink-0" />
					<div class="text-sm text-yellow-700 dark:text-yellow-400">
						<span class="font-medium">{formatKeystroke(capturedKey)}</span> conflicts with common terminal shortcuts. It may not work as expected in all terminals.
					</div>
				</div>
			{/if}

			{#if conflicts.length > 0 && !isReserved}
				<div class="flex items-start gap-2 p-3 rounded-lg bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800">
					<AlertTriangle class="w-4 h-4 text-yellow-500 mt-0.5 flex-shrink-0" />
					<div class="text-sm text-yellow-700 dark:text-yellow-400">
						<p class="font-medium mb-1">Key conflict detected:</p>
						{#each conflicts as conflict}
							<p>
								<span class="font-mono">{formatKeystroke(conflict.key)}</span> is already bound to
								<span class="font-medium">{conflict.existingActionLabel}</span>
								in {conflict.context} context
							</p>
						{/each}
					</div>
				</div>
			{/if}
		</div>

		<!-- Footer -->
		<div class="flex items-center justify-between px-5 py-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50">
			<button onclick={handleClear} class="btn btn-ghost text-sm" disabled={!capturedKey}>
				Clear
			</button>
			<div class="flex items-center gap-2">
				<button onclick={oncancel} class="btn btn-ghost text-sm">
					Cancel
				</button>
				<button
					onclick={handleConfirm}
					class="btn btn-primary text-sm"
					disabled={!capturedKey || isReserved || isWaitingForChord}
				>
					Confirm
				</button>
			</div>
		</div>
	</div>
</div>
