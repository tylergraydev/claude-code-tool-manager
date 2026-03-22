<script lang="ts">
	import { containerLibrary } from '$lib/stores';
	import type { ContainerTemplate } from '$lib/types';
	import ContainerForm from './ContainerForm.svelte';
	import { ArrowLeft, Plus } from 'lucide-svelte';

	type Props = {
		onSubmit: (values: any) => void;
		onCancel: () => void;
	};

	let { onSubmit, onCancel }: Props = $props();

	let step = $state<'pick' | 'configure'>('pick');
	let selectedTemplate = $state<ContainerTemplate | null>(null);
	let selectedCategory = $state('all');

	const iconMap: Record<string, string> = {
		'ubuntu-dev': '/icons/templates/ubuntu.svg',
		'node-dev': '/icons/templates/nodejs.svg',
		'typescript-fullstack': '/icons/templates/typescript.svg',
		'rust-tauri-dev': '/icons/templates/rust.svg',
		'python-dev': '/icons/templates/python.svg',
		'go-dev': '/icons/templates/go.svg',
		'dotnet-dev': '/icons/templates/dotnet.svg',
		'postgres': '/icons/templates/postgresql.svg',
		'redis': '/icons/templates/redis.svg',
	};

	function getIcon(template: ContainerTemplate): string | null {
		return iconMap[template.id] || null;
	}

	const categories = $derived.by(() => {
		const cats = new Set<string>();
		for (const t of containerLibrary.templates) {
			if (t.category) cats.add(t.category);
		}
		return Array.from(cats).sort();
	});

	const filteredTemplates = $derived(
		selectedCategory === 'all'
			? containerLibrary.templates
			: containerLibrary.templates.filter((t) => t.category === selectedCategory)
	);

	function selectTemplate(template: ContainerTemplate) {
		selectedTemplate = template;
		step = 'configure';
	}

	function selectCustom() {
		selectedTemplate = null;
		step = 'configure';
	}

	function goBack() {
		step = 'pick';
		selectedTemplate = null;
	}

	const prefill = $derived(selectedTemplate ? {
		description: selectedTemplate.description,
		containerType: 'docker',
		image: selectedTemplate.image,
		workingDir: selectedTemplate.workingDir,
		dockerfile: selectedTemplate.dockerfile,
		postCreateCommand: selectedTemplate.postCreateCommand,
		postStartCommand: selectedTemplate.postStartCommand,
		icon: selectedTemplate.icon,
		ports: selectedTemplate.ports,
		volumes: selectedTemplate.volumes,
		env: selectedTemplate.env,
	} : undefined);
</script>

{#if step === 'pick'}
	<div class="space-y-4">
		<p class="text-sm text-gray-500 dark:text-gray-400">Choose a template to get started quickly, or create a custom container from scratch.</p>

		<!-- Custom option -->
		<button
			onclick={selectCustom}
			class="w-full p-4 rounded-xl border-2 border-dashed border-gray-300 dark:border-gray-600 hover:border-primary-400 dark:hover:border-primary-500 transition-colors text-left group"
		>
			<div class="flex items-center gap-3">
				<div class="w-10 h-10 rounded-lg bg-gray-100 dark:bg-gray-700 flex items-center justify-center group-hover:bg-primary-50 dark:group-hover:bg-primary-900/30 transition-colors">
					<Plus class="w-5 h-5 text-gray-400 group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors" />
				</div>
				<div>
					<p class="font-medium text-gray-900 dark:text-white">Custom Container</p>
					<p class="text-sm text-gray-500 dark:text-gray-400">Start from scratch with a blank configuration</p>
				</div>
			</div>
		</button>

		<!-- Category filter -->
		{#if containerLibrary.templates.length > 0}
			<div class="flex gap-2 flex-wrap">
				<button
					class="px-3 py-1.5 text-sm rounded-md border transition-colors {selectedCategory === 'all'
						? 'bg-primary-50 dark:bg-primary-900/30 border-primary-300 dark:border-primary-700 text-primary-700 dark:text-primary-300'
						: 'border-gray-200 dark:border-gray-600 text-gray-600 dark:text-gray-400 hover:border-gray-300 dark:hover:border-gray-500'}"
					onclick={() => selectedCategory = 'all'}
				>All</button>
				{#each categories as cat}
					<button
						class="px-3 py-1.5 text-sm rounded-md border transition-colors {selectedCategory === cat
							? 'bg-primary-50 dark:bg-primary-900/30 border-primary-300 dark:border-primary-700 text-primary-700 dark:text-primary-300'
							: 'border-gray-200 dark:border-gray-600 text-gray-600 dark:text-gray-400 hover:border-gray-300 dark:hover:border-gray-500'}"
						onclick={() => selectedCategory = cat}
					>{cat}</button>
				{/each}
			</div>

			<!-- Template grid -->
			<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
				{#each filteredTemplates as template (template.id)}
					<button
						onclick={() => selectTemplate(template)}
						class="card p-4 text-left hover:shadow-md hover:border-primary-300 dark:hover:border-primary-600 transition-all"
					>
						<div class="flex items-start gap-3">
							{#if getIcon(template)}
								<img src={getIcon(template)} alt="{template.name} logo" class="w-8 h-8 shrink-0" />
							{:else}
								<div class="text-2xl shrink-0">{template.icon || '\u{1F4E6}'}</div>
							{/if}
							<div class="min-w-0">
								<p class="font-medium text-gray-900 dark:text-white">{template.name}</p>
								<p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5 line-clamp-2">{template.description}</p>
								<p class="text-xs text-gray-400 dark:text-gray-500 mt-1.5 font-mono truncate">{template.image}</p>
							</div>
						</div>
					</button>
				{/each}
			</div>
		{:else}
			<p class="text-sm text-gray-500 dark:text-gray-400 py-4 text-center">Loading templates...</p>
		{/if}
	</div>
{:else}
	<div class="space-y-4">
		<!-- Back + context -->
		<div class="flex items-center gap-3">
			<button onclick={goBack} class="btn btn-ghost p-1.5" aria-label="Back to templates">
				<ArrowLeft class="w-4 h-4" />
			</button>
			{#if selectedTemplate}
				<div class="flex items-center gap-2">
					{#if getIcon(selectedTemplate)}
						<img src={getIcon(selectedTemplate)} alt="{selectedTemplate.name} logo" class="w-5 h-5" />
					{:else}
						<span class="text-lg">{selectedTemplate.icon}</span>
					{/if}
					<span class="text-sm font-medium text-gray-900 dark:text-white">{selectedTemplate.name}</span>
					<span class="text-xs text-gray-400 dark:text-gray-500">— customize below</span>
				</div>
			{:else}
				<span class="text-sm font-medium text-gray-900 dark:text-white">Custom Container</span>
			{/if}
		</div>

		<ContainerForm container={prefill} {onSubmit} {onCancel} />
	</div>
{/if}
