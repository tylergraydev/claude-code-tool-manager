<script lang="ts">
	import type { ContainerTemplate } from '$lib/types';

	let { template, onUse }: {
		template: ContainerTemplate;
		onUse: (template: ContainerTemplate) => void;
	} = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="card hover:shadow-md transition-shadow duration-200 cursor-pointer" onclick={() => onUse(template)}>
	<div class="flex items-start gap-3">
		<span class="text-2xl">{template.icon}</span>
		<div class="flex-1 min-w-0">
			<h3 class="font-medium text-gray-900 dark:text-white">{template.name}</h3>
			<p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">{template.description}</p>
			<div class="flex items-center gap-2 mt-2">
				<span class="text-xs px-2 py-0.5 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400">{template.category}</span>
				<span class="text-xs text-gray-400">{template.image}</span>
			</div>
		</div>
	</div>
</div>
