<script lang="ts">
	import type { ClaudeSettings } from '$lib/types';
	import { Save } from 'lucide-svelte';

	type Props = {
		settings: ClaudeSettings;
		onsave: (settings: ClaudeSettings) => void;
	};

	let { settings, onsave }: Props = $props();

	let disableAutoMode = $state<boolean | undefined>(settings.disableAutoMode);
	let environment = $state(settings.autoModeEnvironment ?? '');
	let allow = $state(settings.autoModeAllow ?? '');
	let softDeny = $state(settings.autoModeSoftDeny ?? '');

	$effect(() => {
		disableAutoMode = settings.disableAutoMode;
		environment = settings.autoModeEnvironment ?? '';
		allow = settings.autoModeAllow ?? '';
		softDeny = settings.autoModeSoftDeny ?? '';
	});

	function handleSave() {
		onsave({
			...settings,
			disableAutoMode,
			autoModeEnvironment: environment.trim() || undefined,
			autoModeAllow: allow.trim() || undefined,
			autoModeSoftDeny: softDeny.trim() || undefined
		});
	}
</script>

<div class="space-y-6">
	<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
		<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">Auto Mode</h3>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
			Configure the <code class="text-xs bg-gray-100 dark:bg-gray-700 px-1 rounded">auto</code> permission mode — what it can do, what it should avoid, and where it runs
		</p>

		<div class="space-y-4">
			<div class="flex items-center justify-between">
				<div>
					<label for="disable-auto" class="text-sm font-medium text-gray-700 dark:text-gray-300">
						Disable Auto Mode
					</label>
					<p class="text-xs text-gray-500 dark:text-gray-400">
						Prevent use of the auto permission mode entirely
					</p>
				</div>
				<label class="relative inline-flex items-center cursor-pointer">
					<input
						id="disable-auto"
						type="checkbox"
						checked={disableAutoMode === true}
						onchange={(e) => {
							const target = e.currentTarget;
							disableAutoMode = target.checked ? true : undefined;
						}}
						class="sr-only peer"
					/>
					<div
						class="w-9 h-5 bg-gray-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-500/40 rounded-full peer dark:bg-gray-600 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all dark:after:border-gray-500 peer-checked:bg-blue-500"
					></div>
				</label>
			</div>

			<div class="border-t border-gray-200 dark:border-gray-700 pt-4">
				<label for="auto-mode-env" class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Environment
				</label>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					Describe the trusted infrastructure this runs on (e.g., CI server, development machine, staging environment)
				</p>
				<textarea
					id="auto-mode-env"
					bind:value={environment}
					rows={3}
					placeholder="e.g., This is a CI server running in a Docker container with no access to production data"
					class="input w-full resize-y text-sm"
				></textarea>
			</div>

			<div>
				<label for="auto-mode-allow" class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Allow
				</label>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					Describe what actions auto mode should allow without prompting
				</p>
				<textarea
					id="auto-mode-allow"
					bind:value={allow}
					rows={3}
					placeholder="e.g., File reads, git operations, running tests, installing npm packages"
					class="input w-full resize-y text-sm"
				></textarea>
			</div>

			<div>
				<label for="auto-mode-soft-deny" class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Soft Deny
				</label>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					Describe what actions auto mode should block (user can still override)
				</p>
				<textarea
					id="auto-mode-soft-deny"
					bind:value={softDeny}
					rows={3}
					placeholder="e.g., Network access to production APIs, deleting files outside the project, modifying system configs"
					class="input w-full resize-y text-sm"
				></textarea>
			</div>
		</div>
	</div>

	<div class="flex justify-end">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save Auto Mode Settings
		</button>
	</div>
</div>
