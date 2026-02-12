<script lang="ts">
	import type { StatusLineSegment, SegmentColor, StatusLineTheme } from '$lib/types';
	import { SEGMENT_COLORS, POWERLINE_DEFAULT_BG } from '$lib/types';

	type Props = {
		segments: StatusLineSegment[];
		theme?: StatusLineTheme;
		padding?: number;
	};

	let { segments, theme = 'default', padding = 0 }: Props = $props();

	const paddingStr = $derived('\u00A0'.repeat(padding));

	function getColorHex(color: SegmentColor | undefined): string {
		const found = SEGMENT_COLORS.find((c) => c.value === color);
		return found?.hex || '#f8fafc';
	}

	function getBgColorHex(color: SegmentColor | undefined): string | undefined {
		if (!color) return undefined;
		const found = SEGMENT_COLORS.find((c) => c.value === color);
		return found?.hex;
	}

	const isPowerline = $derived(theme === 'powerline' || theme === 'powerline_round');
	// Use CSS triangles in preview since Powerline glyphs need patched fonts
	const isRound = $derived(theme === 'powerline_round');

	interface SegmentPreview {
		label: string;
		bar?: { filled: number; total: number; color: string };
		suffix?: string;
	}

	function getSegmentPreview(seg: StatusLineSegment): SegmentPreview {
		const label = seg.label ? `${seg.label} ` : '';
		const color = getColorHex(seg.color);
		switch (seg.type) {
			case 'model':
				return { label: label + (seg.format === 'full' ? 'claude-opus-4-6' : 'opus') };
			case 'cost':
				return { label: label + (seg.format?.includes('0000') ? '$1.2345' : '$1.23') };
			case 'context':
				if (seg.format === 'fraction') return { label: label + '156k/200k' };
				if (seg.format === 'bar') return { label: label, bar: { filled: 4, total: 6, color }, suffix: ' 78%' };
				return { label: label + '78%' };
			case 'context_remaining':
				if (seg.format === 'bar') return { label: label, bar: { filled: 2, total: 6, color }, suffix: ' 22%' };
				return { label: label + '22%' };
			case 'cwd':
				if (seg.format === 'full') return { label: label + '/home/user/project' };
				if (seg.format === 'short') return { label: label + '~/project' };
				return { label: label + 'project' };
			case 'project_dir':
				if (seg.format === 'full') return { label: label + '/home/user/project' };
				return { label: label + 'project' };
			case 'tokens_in':
				return { label: label + (seg.format === 'full' ? '156000' : '156k') };
			case 'tokens_out':
				return { label: label + (seg.format === 'full' ? '12000' : '12k') };
			case 'duration':
				return { label: label + (seg.format === 'hms' ? '0:05:30' : '5m 30s') };
			case 'api_duration':
				return { label: label + (seg.format === 'hms' ? '0:02:10' : '2m 10s') };
			case 'lines_changed':
				return { label: label + (seg.format === 'net' ? '+133' : '+156 -23') };
			case 'git_branch':
				return { label: label + 'main' };
			case 'git_status':
				return { label: label + (seg.format === 'verbose' ? '3 staged, 5 modified' : '+3 ~5') };
			case 'session_id':
				return { label: label + (seg.format === 'full' ? 'a1b2c3d4-e5f6-7890' : 'a1b2c3d4') };
			case 'version':
				return { label: label + 'v1.0.80' };
			case 'agent_name':
				return { label: label + 'security-reviewer' };
			case 'five_hour_usage':
				if (seg.format === 'bar') return { label: label, bar: { filled: 1, total: 6, color }, suffix: ' 12%' };
				if (seg.format === 'percent_only') return { label: label + '12%' };
				return { label: label + '12% 3h20m' };
			case 'weekly_usage':
				if (seg.format === 'bar') return { label: label, bar: { filled: 3, total: 6, color }, suffix: ' 45%' };
				if (seg.format === 'percent_only') return { label: label + '45%' };
				return { label: label + '45% wk 85%' };
			case 'vim_mode':
				return { label: label + 'NORMAL' };
			case 'separator':
				return { label: seg.separatorChar || '|' };
			case 'custom_text':
				return { label: seg.customText || 'text' };
			default:
				return { label: '???' };
		}
	}

	/** Get the effective bg color for a segment (explicit or powerline default) */
	function getEffectiveBg(seg: StatusLineSegment): SegmentColor | undefined {
		if (seg.bgColor) return seg.bgColor;
		if (isPowerline) {
			return (POWERLINE_DEFAULT_BG[seg.type] as SegmentColor) || 'gray';
		}
		return undefined;
	}

	const enabledSegments = $derived(
		segments.filter((s) => s.enabled).sort((a, b) => a.position - b.position)
	);

	/** For Powerline mode, filter out separators and line_breaks */
	const powerlineSegments = $derived(
		enabledSegments.filter((s) => s.type !== 'separator' && s.type !== 'line_break')
	);

	// Split segments into lines at line_break boundaries (default mode)
	const lines = $derived.by(() => {
		const result: StatusLineSegment[][] = [[]];
		for (const seg of enabledSegments) {
			if (seg.type === 'line_break') {
				result.push([]);
			} else {
				result[result.length - 1].push(seg);
			}
		}
		return result;
	});
</script>

<div class="bg-gray-900 rounded-lg px-4 py-2.5 font-mono text-sm overflow-x-auto">
	{#if enabledSegments.length === 0}
		<span class="text-gray-500 italic">No segments — add some above</span>
	{:else if isPowerline}
		<!-- Powerline Mode Preview -->
		<div class="flex items-stretch" style="height: 28px">
			{#if padding > 0}<span class="inline-flex items-center text-gray-900">{paddingStr}</span>{/if}
			{#each powerlineSegments as seg, idx (seg.id)}
				{@const preview = getSegmentPreview(seg)}
				{@const bgColor = getEffectiveBg(seg)}
				{@const bgHex = getBgColorHex(bgColor) || '#374151'}
				{@const fgHex = getColorHex(seg.color)}
				{@const nextBgHex = idx < powerlineSegments.length - 1
					? (getBgColorHex(getEffectiveBg(powerlineSegments[idx + 1])) || '#374151')
					: '#111827'}
				<!-- Segment content with bg -->
				<span
					class="inline-flex items-center px-2 whitespace-nowrap"
					style="background-color: {bgHex}; color: {fgHex}"
				>
					{#if preview.label}{preview.label}{/if}
					{#if preview.bar}
						<span class="inline-flex items-center mx-0.5">[<!--
							-->{#each Array(preview.bar.total) as _, i}
								<span
									class="inline-block w-1.5 h-3"
									style="background-color: {i < preview.bar.filled ? preview.bar.color : '#555'}"
								></span>
							{/each}<!--
						-->]</span>
					{/if}
					{#if preview.suffix}{preview.suffix}{/if}
				</span>
				<!-- Powerline arrow separator using inline SVG -->
				<svg class="shrink-0" style="display: block; height: 100%; width: 14px;" viewBox="0 0 14 28" preserveAspectRatio="none">
					{#if isRound}
						<rect width="14" height="28" fill={nextBgHex} />
						<path d="M0,0 C10,0 14,14 14,14 C14,14 10,28 0,28 Z" fill={bgHex} />
					{:else}
						<rect width="14" height="28" fill={nextBgHex} />
						<path d="M0,0 L14,14 L0,28 Z" fill={bgHex} />
					{/if}
				</svg>
			{/each}
		</div>
	{:else}
		<!-- Default Mode Preview -->
		{#each lines as line, lineIdx}
			<div class="flex items-center gap-1.5" class:mt-1={lineIdx > 0}>
				{#if padding > 0}<span class="text-gray-900">{paddingStr}</span>{/if}
				{#each line as seg (seg.id)}
					{@const preview = getSegmentPreview(seg)}
					{@const bgColor = getEffectiveBg(seg)}
					{@const bgHex = getBgColorHex(bgColor)}
					<span
						style="color: {getColorHex(seg.color)}{bgHex ? `; background-color: ${bgHex}; padding: 0 4px; border-radius: 2px` : ''}"
						class="inline-flex items-center"
					>
						{#if preview.label}{preview.label}{/if}
						{#if preview.bar}
							<span class="inline-flex items-center mx-0.5">[<!--
								-->{#each Array(preview.bar.total) as _, i}
									<span
										class="inline-block w-1.5 h-3"
										style="background-color: {i < preview.bar.filled ? preview.bar.color : '#555'}"
									></span>
								{/each}<!--
							-->]</span>
						{/if}
						{#if preview.suffix}{preview.suffix}{/if}
					</span>
				{/each}
			</div>
		{/each}
	{/if}
</div>
