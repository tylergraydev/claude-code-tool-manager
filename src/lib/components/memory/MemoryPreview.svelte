<script lang="ts">
	type Props = {
		html: string;
	};

	let { html }: Props = $props();
</script>

<div
	class="w-full h-full min-h-[400px] p-6 overflow-auto
		bg-white dark:bg-gray-900
		border border-gray-200 dark:border-gray-700
		rounded-lg prose-container"
>
	{#if html}
		{@html html}
	{:else}
		<p class="text-gray-400 dark:text-gray-500 italic">Nothing to preview</p>
	{/if}
</div>

<style>
	.prose-container :global(h1) {
		font-size: 1.5rem;
		font-weight: 700;
		margin-bottom: 0.75rem;
		margin-top: 1.25rem;
		color: var(--text-primary, #111827);
	}
	.prose-container :global(h2) {
		font-size: 1.25rem;
		font-weight: 600;
		margin-bottom: 0.5rem;
		margin-top: 1rem;
		color: var(--text-primary, #111827);
	}
	.prose-container :global(h3) {
		font-size: 1.1rem;
		font-weight: 600;
		margin-bottom: 0.5rem;
		margin-top: 0.75rem;
		color: var(--text-primary, #111827);
	}
	.prose-container :global(p) {
		margin-bottom: 0.5rem;
		line-height: 1.6;
		color: var(--text-secondary, #374151);
	}
	.prose-container :global(ul),
	.prose-container :global(ol) {
		margin-bottom: 0.5rem;
		padding-left: 1.5rem;
		color: var(--text-secondary, #374151);
	}
	.prose-container :global(ul) {
		list-style-type: disc;
	}
	.prose-container :global(ol) {
		list-style-type: decimal;
	}
	.prose-container :global(li) {
		margin-bottom: 0.25rem;
		line-height: 1.5;
	}
	.prose-container :global(code) {
		background: #f3f4f6;
		padding: 0.15rem 0.35rem;
		border-radius: 0.25rem;
		font-size: 0.875rem;
		font-family: ui-monospace, monospace;
	}
	.prose-container :global(pre) {
		background: #1f2937;
		color: #e5e7eb;
		padding: 1rem;
		border-radius: 0.5rem;
		margin-bottom: 0.75rem;
		overflow-x: auto;
	}
	.prose-container :global(pre code) {
		background: transparent;
		padding: 0;
		color: inherit;
	}
	.prose-container :global(a) {
		color: #3b82f6;
		text-decoration: underline;
	}
	.prose-container :global(strong) {
		font-weight: 700;
	}
	.prose-container :global(em) {
		font-style: italic;
	}
	.prose-container :global(blockquote) {
		border-left: 3px solid #d1d5db;
		padding-left: 1rem;
		margin-bottom: 0.5rem;
		color: #6b7280;
	}
	.prose-container :global(table) {
		width: 100%;
		border-collapse: collapse;
		margin-bottom: 0.75rem;
	}
	.prose-container :global(th),
	.prose-container :global(td) {
		border: 1px solid #d1d5db;
		padding: 0.5rem 0.75rem;
		text-align: left;
	}
	.prose-container :global(th) {
		background: #f3f4f6;
		font-weight: 600;
	}
	.prose-container :global(hr) {
		border: none;
		border-top: 1px solid #d1d5db;
		margin: 1rem 0;
	}
	.prose-container :global(input[type='checkbox']) {
		margin-right: 0.5rem;
	}

	:global(.dark) .prose-container :global(h1),
	:global(.dark) .prose-container :global(h2),
	:global(.dark) .prose-container :global(h3) {
		color: #f9fafb;
	}
	:global(.dark) .prose-container :global(p),
	:global(.dark) .prose-container :global(ul),
	:global(.dark) .prose-container :global(ol),
	:global(.dark) .prose-container :global(li) {
		color: #d1d5db;
	}
	:global(.dark) .prose-container :global(code) {
		background: #374151;
		color: #e5e7eb;
	}
	:global(.dark) .prose-container :global(pre) {
		background: #111827;
	}
	:global(.dark) .prose-container :global(blockquote) {
		border-color: #4b5563;
		color: #9ca3af;
	}
	:global(.dark) .prose-container :global(th) {
		background: #374151;
		color: #e5e7eb;
	}
	:global(.dark) .prose-container :global(th),
	:global(.dark) .prose-container :global(td) {
		border-color: #4b5563;
	}
	:global(.dark) .prose-container :global(hr) {
		border-color: #4b5563;
	}
</style>
