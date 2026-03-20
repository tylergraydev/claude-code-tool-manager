<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { Container } from 'lucide-svelte';
</script>

<Header title="Containers" subtitle="Manage Docker containers for your projects" />

<div class="flex-1 overflow-auto p-6">
	<div class="text-center py-16 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
		<Container class="w-16 h-16 mx-auto text-gray-400 mb-4" />
		<h2 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">Coming Soon</h2>
		<p class="text-gray-500 dark:text-gray-400 max-w-md mx-auto">
			Container management is under development. You'll be able to manage Docker containers,
			images, and dev environments directly from here.
		</p>
	</div>
</div>
