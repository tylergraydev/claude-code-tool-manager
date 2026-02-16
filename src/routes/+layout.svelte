<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { Sidebar } from '$lib/components/layout';
	import { Toast } from '$lib/components/shared';
	import UpdateNotification from '$lib/components/shared/UpdateNotification.svelte';
	import WhatsNewModal from '$lib/components/shared/WhatsNewModal.svelte';
	import { mcpLibrary, projectsStore, skillLibrary, subagentLibrary, hookLibrary, commandLibrary, statuslineLibrary, spinnerVerbLibrary, whatsNew, debugStore } from '$lib/stores';
	import { installDebugInterceptor } from '$lib/utils/debugLogger';

	let { children } = $props();

	onMount(async () => {
		// Load initial data
		await Promise.all([
			mcpLibrary.load(),
			projectsStore.loadProjects(),
			projectsStore.loadGlobalMcps(),
			skillLibrary.load(),
			skillLibrary.loadGlobalSkills(),
			subagentLibrary.load(),
			subagentLibrary.loadGlobalSubAgents(),
			hookLibrary.load(),
			hookLibrary.loadGlobalHooks(),
			commandLibrary.load(),
			commandLibrary.loadGlobalCommands(),
			statuslineLibrary.load(),
			spinnerVerbLibrary.load()
		]);

		// Load debug state and install interceptor if enabled
		await debugStore.load();
		if (debugStore.isEnabled) {
			installDebugInterceptor();
			console.log('[Debug] App started with debug mode enabled');
		}

		// Check for "What's New" after update (with delay to not block startup)
		setTimeout(() => {
			whatsNew.checkForWhatsNew();
		}, 1500);
	});
</script>

<div class="flex h-screen overflow-hidden bg-gray-50 dark:bg-gray-900">
	<Sidebar />

	<main class="flex-1 flex flex-col overflow-hidden">
		{@render children()}
	</main>
</div>

<Toast />
<UpdateNotification />
<WhatsNewModal />
