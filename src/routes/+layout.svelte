<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { Sidebar } from '$lib/components/layout';
	import { Toast } from '$lib/components/shared';
	import UpdateNotification from '$lib/components/shared/UpdateNotification.svelte';
	import WhatsNewModal from '$lib/components/shared/WhatsNewModal.svelte';
	import { mcpLibrary, projectsStore, skillLibrary, subagentLibrary, hookLibrary, commandLibrary, containerLibrary, statuslineLibrary, spinnerVerbLibrary, ruleLibrary, whatsNew, debugStore } from '$lib/stores';
	import { installDebugInterceptor } from '$lib/utils/debugLogger';

	let { children } = $props();

	onMount(() => {
		// Priority 1: Load stores needed for dashboard immediately
		Promise.all([
			mcpLibrary.load(),
			projectsStore.loadProjects(),
			projectsStore.loadGlobalMcps()
		]).catch((e) => {
			console.error('[Layout] Failed to load core stores:', e);
		});

		// Priority 2: Defer secondary stores to reduce IPC congestion
		requestAnimationFrame(() => {
			Promise.all([
				skillLibrary.load(),
				skillLibrary.loadGlobalSkills(),
				subagentLibrary.load(),
				subagentLibrary.loadGlobalSubAgents(),
				hookLibrary.load(),
				hookLibrary.loadGlobalHooks(),
				commandLibrary.load(),
				commandLibrary.loadGlobalCommands(),
				ruleLibrary.load(),
				ruleLibrary.loadGlobalRules()
			]).catch((e) => {
				console.error('[Layout] Failed to load secondary stores:', e);
			});
		});

		// Priority 3: Low-priority stores after initial paint
		setTimeout(() => {
			statuslineLibrary.load();
			spinnerVerbLibrary.load();
			containerLibrary.load();
		}, 100);

		// Load debug state and install interceptor if enabled
		debugStore.load().then(() => {
			if (debugStore.isEnabled) {
				installDebugInterceptor();
			}
		});

		// Check for "What's New" after update (with delay to not block startup)
		setTimeout(() => {
			whatsNew.checkForWhatsNew();
		}, 1500);
	});
</script>

<a href="#main-content" class="skip-to-content">Skip to main content</a>

<div class="flex h-screen overflow-hidden bg-gray-50 dark:bg-gray-900">
	<Sidebar />

	<main id="main-content" class="flex-1 flex flex-col overflow-hidden" tabindex="-1">
		{@render children()}
	</main>
</div>

<Toast />
<UpdateNotification />
<WhatsNewModal />
