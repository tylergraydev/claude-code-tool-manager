<script lang="ts">
	import { goto } from '$app/navigation';
	import { onboarding } from '$lib/stores/onboarding.svelte';
	import { FolderOpen, Plug, X, Check, Settings, ChevronRight } from 'lucide-svelte';
	import type { OnboardingStep } from '$lib/stores/onboarding.svelte';

	type Props = {
		projectCount: number;
		mcpCount: number;
		globalMcpCount: number;
	};

	let { projectCount, mcpCount, globalMcpCount }: Props = $props();

	// Sync completion state with actual data
	$effect(() => {
		onboarding.syncWithStores(projectCount, mcpCount, globalMcpCount);
	});

	interface Step {
		id: OnboardingStep;
		label: string;
		description: string;
		href: string;
		icon: any;
	}

	const steps: Step[] = [
		{
			id: 'add-project',
			label: 'Add a project',
			description: 'Point to a folder where you use Claude Code',
			href: '/projects',
			icon: FolderOpen
		},
		{
			id: 'add-mcp',
			label: 'Add an MCP server',
			description: 'Create or import an MCP to your library',
			href: '/library',
			icon: Plug
		},
		{
			id: 'assign-mcp',
			label: 'Enable a global MCP',
			description: 'Turn on an MCP across all projects',
			href: '/library',
			icon: Check
		},
		{
			id: 'explore-settings',
			label: 'Explore settings',
			description: 'Configure themes, keybindings, and more',
			href: '/settings',
			icon: Settings
		}
	];

	function isStepComplete(stepId: OnboardingStep): boolean {
		return onboarding.completedSteps.includes(stepId);
	}

	function handleStepClick(step: Step) {
		goto(step.href);
	}

	function handleDismiss() {
		onboarding.dismiss();
	}
</script>

<section class="space-y-6">
	<!-- Welcome banner -->
	<div class="relative overflow-hidden rounded-xl border border-primary-200 dark:border-primary-800/50 bg-gradient-to-br from-primary-50 via-white to-blue-50 dark:from-primary-950/40 dark:via-gray-800 dark:to-gray-800 p-6">
		<button
			onclick={handleDismiss}
			class="absolute top-3 right-3 p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded-lg hover:bg-white/60 dark:hover:bg-gray-700/60 transition-colors"
			aria-label="Dismiss getting started guide"
		>
			<X class="w-4 h-4" />
		</button>

		<div class="max-w-xl">
			{#if onboarding.isFirstRun}
				<p class="text-sm font-medium text-primary-600 dark:text-primary-400 mb-1">Welcome to Tool Manager</p>
				<h2 class="text-lg font-semibold text-gray-900 dark:text-white">
					Manage your Claude Code setup visually
				</h2>
				<p class="text-sm text-gray-600 dark:text-gray-400 mt-1.5 leading-relaxed">
					Add your project folders, configure MCP servers, and keep everything in sync — no more editing JSON by hand.
				</p>
			{:else}
				<p class="text-sm font-medium text-primary-600 dark:text-primary-400 mb-1">Getting started</p>
				<h2 class="text-lg font-semibold text-gray-900 dark:text-white">
					Finish setting up
				</h2>
				<p class="text-sm text-gray-600 dark:text-gray-400 mt-1.5">
					Complete these steps to get the most out of Tool Manager.
				</p>
			{/if}
		</div>

		<!-- Progress bar -->
		<div class="mt-4 flex items-center gap-3">
			<div
				class="flex-1 h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden"
				role="progressbar"
				aria-valuenow={onboarding.completedSteps.length}
				aria-valuemin={0}
				aria-valuemax={steps.length}
				aria-label="Setup progress"
			>
				<div
					class="h-full bg-primary-500 rounded-full transition-all duration-500 ease-out"
					style="width: {onboarding.progress * 100}%"
				></div>
			</div>
			<span class="text-xs font-medium text-gray-500 dark:text-gray-400 tabular-nums">
				{onboarding.completedSteps.length}/{steps.length}
			</span>
		</div>
	</div>

	<!-- Checklist -->
	<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
		{#each steps as step (step.id)}
			{@const done = isStepComplete(step.id)}
			<button
				onclick={() => handleStepClick(step)}
				class="group flex items-start gap-3 rounded-xl border p-4 text-left transition-all
					{done
						? 'border-green-200 bg-green-50/50 dark:border-green-900/40 dark:bg-green-950/20'
						: 'border-gray-200 bg-white hover:border-primary-300 hover:shadow-sm dark:border-gray-700 dark:bg-gray-800 dark:hover:border-primary-700'}"
			>
				<!-- Status indicator -->
				<div class="flex-shrink-0 mt-0.5">
					{#if done}
						<div class="w-7 h-7 rounded-lg bg-green-500 flex items-center justify-center">
							<Check class="w-4 h-4 text-white" />
						</div>
					{:else}
						<div class="w-7 h-7 rounded-lg bg-gray-100 dark:bg-gray-700 flex items-center justify-center group-hover:bg-primary-100 dark:group-hover:bg-primary-900/40 transition-colors">
							<step.icon class="w-4 h-4 text-gray-400 group-hover:text-primary-500 transition-colors" />
						</div>
					{/if}
				</div>

				<div class="flex-1 min-w-0">
					<p class="text-sm font-medium {done ? 'text-green-700 dark:text-green-400 line-through decoration-green-400/40' : 'text-gray-900 dark:text-white'}">
						{step.label}
					</p>
					<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
						{step.description}
					</p>
				</div>

				{#if !done}
					<ChevronRight class="w-4 h-4 text-gray-300 dark:text-gray-600 group-hover:text-primary-400 transition-colors flex-shrink-0 mt-1" />
				{/if}
			</button>
		{/each}
	</div>
</section>
