<script lang="ts">
	import type { StatusLineSegment, SegmentColor } from '$lib/types';
	import { SEGMENT_TYPES, SEGMENT_COLORS } from '$lib/types';

	type Props = {
		segment: StatusLineSegment;
		onChange: (updated: StatusLineSegment) => void;
	};

	let { segment, onChange }: Props = $props();

	const meta = $derived(SEGMENT_TYPES.find((t) => t.type === segment.type));

	function update(field: keyof StatusLineSegment, value: unknown) {
		onChange({ ...segment, [field]: value });
	}
</script>

<div class="space-y-4">
	<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">
		Configure: {meta?.label || segment.type}
	</h4>

	<!-- Foreground Color -->
	<div>
		<label class="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Text Color</label>
		<div class="flex flex-wrap gap-1.5">
			{#each SEGMENT_COLORS as c}
				<button
					onclick={() => update('color', c.value)}
					class="w-6 h-6 rounded-full border-2 transition-transform hover:scale-110
						{segment.color === c.value ? 'border-primary-500 scale-110' : 'border-transparent'}"
					style="background-color: {c.hex}"
					title={c.label}
				></button>
			{/each}
		</div>
	</div>

	<!-- Background Color -->
	{#if segment.type !== 'separator' && segment.type !== 'line_break'}
		<div>
			<label class="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Background Color</label>
			<div class="flex flex-wrap gap-1.5 items-center">
				<button
					onclick={() => update('bgColor', undefined)}
					class="w-6 h-6 rounded-full border-2 transition-transform hover:scale-110 flex items-center justify-center text-[10px]
						{!segment.bgColor ? 'border-primary-500 scale-110' : 'border-gray-300 dark:border-gray-600'}"
					style="background: repeating-conic-gradient(#808080 0% 25%, transparent 0% 50%) 50% / 8px 8px"
					title="None"
				></button>
				{#each SEGMENT_COLORS as c}
					<button
						onclick={() => update('bgColor', c.value)}
						class="w-6 h-6 rounded-full border-2 transition-transform hover:scale-110
							{segment.bgColor === c.value ? 'border-primary-500 scale-110' : 'border-transparent'}"
						style="background-color: {c.hex}"
						title={c.label}
					></button>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Format (if applicable) -->
	{#if meta?.formats && meta.formats.length > 0}
		<div>
			<label class="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Format</label>
			<select
				value={segment.format || meta.formats[0].value}
				onchange={(e) => update('format', (e.target as HTMLSelectElement).value)}
				class="w-full px-3 py-1.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm text-gray-900 dark:text-white"
			>
				{#each meta.formats as fmt}
					<option value={fmt.value}>{fmt.label}</option>
				{/each}
			</select>
		</div>
	{/if}

	<!-- Label -->
	{#if segment.type !== 'separator' && segment.type !== 'custom_text'}
		<div>
			<label class="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Label prefix</label>
			<input
				type="text"
				value={segment.label || ''}
				oninput={(e) => update('label', (e.target as HTMLInputElement).value || undefined)}
				placeholder="e.g. Model:"
				class="w-full px-3 py-1.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm text-gray-900 dark:text-white placeholder-gray-400"
			/>
		</div>
	{/if}

	<!-- Separator char -->
	{#if segment.type === 'separator'}
		<div>
			<label class="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Character</label>
			<div class="flex gap-2">
				{#each ['|', '/', '>', '\u2022', '\u2502'] as ch}
					<button
						onclick={() => update('separatorChar', ch)}
						class="w-8 h-8 flex items-center justify-center rounded border text-sm font-mono
							{segment.separatorChar === ch
								? 'border-primary-400 bg-primary-50 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300'
								: 'border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700'}"
					>
						{ch}
					</button>
				{/each}
				<input
					type="text"
					maxlength="3"
					value={segment.separatorChar || '|'}
					oninput={(e) => update('separatorChar', (e.target as HTMLInputElement).value)}
					class="w-12 px-2 py-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded text-sm text-center font-mono text-gray-900 dark:text-white"
				/>
			</div>
		</div>
	{/if}

	<!-- Custom text -->
	{#if segment.type === 'custom_text'}
		<div>
			<label class="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Text</label>
			<input
				type="text"
				value={segment.customText || ''}
				oninput={(e) => update('customText', (e.target as HTMLInputElement).value)}
				placeholder="Enter custom text..."
				class="w-full px-3 py-1.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-sm text-gray-900 dark:text-white placeholder-gray-400"
			/>
		</div>
	{/if}
</div>
