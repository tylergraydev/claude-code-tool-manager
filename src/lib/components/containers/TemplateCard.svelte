<script lang="ts">
	type Template = {
		id: string;
		name: string;
		description: string;
		category: string;
		image: string;
		icon: string;
	};

	type Props = {
		template: Template;
		onUse: (template: Template) => void;
	};

	let { template, onUse }: Props = $props();
</script>

<div class="card hover:shadow-md transition-shadow duration-200 p-4">
	<div class="flex items-start gap-3">
		<div class="text-2xl">{template.icon || '\u{1F4E6}'}</div>
		<div class="flex-1 min-w-0">
			<h3 class="font-medium text-gray-900 dark:text-white">{template.name}</h3>
			<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{template.description}</p>
			<div class="flex items-center gap-2 mt-2">
				<span class="text-xs bg-gray-100 dark:bg-gray-700 px-2 py-0.5 rounded">{template.category}</span>
				<span class="text-xs text-gray-400 font-mono">{template.image}</span>
			</div>
		</div>
		<button onclick={() => onUse(template)} class="btn btn-primary text-sm">
			Use Template
		</button>
	</div>
</div>
