export type StatusLineType = 'custom' | 'premade' | 'raw';

export type SegmentType =
	| 'model'
	| 'cost'
	| 'context'
	| 'context_remaining'
	| 'cwd'
	| 'project_dir'
	| 'tokens_in'
	| 'tokens_out'
	| 'duration'
	| 'api_duration'
	| 'lines_changed'
	| 'git_branch'
	| 'git_status'
	| 'session_id'
	| 'version'
	| 'agent_name'
	| 'five_hour_usage'
	| 'weekly_usage'
	| 'vim_mode'
	| 'separator'
	| 'line_break'
	| 'custom_text';

export type SegmentColor =
	| 'red'
	| 'green'
	| 'yellow'
	| 'blue'
	| 'magenta'
	| 'cyan'
	| 'white'
	| 'bright_red'
	| 'bright_green'
	| 'bright_yellow'
	| 'bright_blue'
	| 'bright_magenta'
	| 'bright_cyan'
	| 'bright_white'
	| 'gray';

export interface StatusLine {
	id: number;
	name: string;
	description: string | null;
	statuslineType: StatusLineType;
	packageName: string | null;
	installCommand: string | null;
	runCommand: string | null;
	rawCommand: string | null;
	padding: number;
	isActive: boolean;
	segmentsJson: string | null;
	generatedScript: string | null;
	icon: string | null;
	author: string | null;
	homepageUrl: string | null;
	tags: string[] | null;
	source: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateStatusLineRequest {
	name: string;
	description?: string | null;
	statuslineType: StatusLineType;
	packageName?: string | null;
	installCommand?: string | null;
	runCommand?: string | null;
	rawCommand?: string | null;
	padding?: number;
	segmentsJson?: string | null;
	generatedScript?: string | null;
	icon?: string | null;
	author?: string | null;
	homepageUrl?: string | null;
	tags?: string[] | null;
}

export type StatusLineTheme = 'default' | 'powerline' | 'powerline_round';

export interface StatusLineSegment {
	id: string;
	type: SegmentType;
	enabled: boolean;
	label?: string;
	format?: string;
	color?: SegmentColor;
	bgColor?: SegmentColor;
	separatorChar?: string;
	customText?: string;
	position: number;
}

/** Wrapper format for segments_json that includes theme info */
export interface SegmentsPayload {
	theme?: StatusLineTheme;
	segments: StatusLineSegment[];
}

/** Parse segments_json which may be a plain array (legacy) or an object with theme */
export function parseSegmentsJson(json: string | null | undefined): SegmentsPayload {
	if (!json) return { theme: 'default', segments: [] };
	try {
		const parsed = JSON.parse(json);
		if (Array.isArray(parsed)) {
			return { theme: 'default', segments: parsed };
		}
		return {
			theme: parsed.theme || 'default',
			segments: parsed.segments || []
		};
	} catch {
		return { theme: 'default', segments: [] };
	}
}

/** Serialize segments with theme into segments_json */
export function serializeSegmentsJson(segments: StatusLineSegment[], theme: StatusLineTheme): string {
	if (theme === 'default') {
		return JSON.stringify(segments);
	}
	return JSON.stringify({ theme, segments });
}

export interface StatusLineGalleryEntry {
	name: string;
	description: string | null;
	author: string | null;
	homepageUrl: string | null;
	installCommand: string | null;
	runCommand: string | null;
	packageName: string | null;
	icon: string | null;
	tags: string[] | null;
	previewText: string | null;
}

// Segment type metadata for UI
export const SEGMENT_TYPES: {
	type: SegmentType;
	label: string;
	description: string;
	defaultColor: SegmentColor;
	formats?: { value: string; label: string }[];
}[] = [
	{
		type: 'model',
		label: 'Model',
		description: 'Current Claude model name',
		defaultColor: 'cyan',
		formats: [
			{ value: 'short', label: 'Short (opus)' },
			{ value: 'full', label: 'Full (claude-opus-4-6)' }
		]
	},
	{
		type: 'cost',
		label: 'Cost',
		description: 'Session cost in USD',
		defaultColor: 'green',
		formats: [
			{ value: '$0.00', label: '$1.23' },
			{ value: '$0.0000', label: '$1.2345' }
		]
	},
	{
		type: 'context',
		label: 'Context',
		description: 'Context window usage percentage',
		defaultColor: 'yellow',
		formats: [
			{ value: 'percentage', label: 'Percentage (78%)' },
			{ value: 'fraction', label: 'Fraction (156k/200k)' },
			{ value: 'bar', label: 'Bar ([████░░] 78%)' }
		]
	},
	{
		type: 'context_remaining',
		label: 'Context Remaining',
		description: 'Remaining context window percentage',
		defaultColor: 'green',
		formats: [
			{ value: 'percentage', label: 'Percentage (22%)' },
			{ value: 'bar', label: 'Bar ([░░████] 22%)' }
		]
	},
	{
		type: 'cwd',
		label: 'Directory',
		description: 'Current working directory',
		defaultColor: 'blue',
		formats: [
			{ value: 'basename', label: 'Basename (project)' },
			{ value: 'short', label: 'Short (~/project)' },
			{ value: 'full', label: 'Full path' }
		]
	},
	{
		type: 'project_dir',
		label: 'Project Dir',
		description: 'Directory where Claude Code was launched',
		defaultColor: 'blue',
		formats: [
			{ value: 'basename', label: 'Basename (project)' },
			{ value: 'full', label: 'Full path' }
		]
	},
	{
		type: 'tokens_in',
		label: 'Tokens In',
		description: 'Total input tokens',
		defaultColor: 'magenta',
		formats: [
			{ value: 'compact', label: 'Compact (156k)' },
			{ value: 'full', label: 'Full (156000)' }
		]
	},
	{
		type: 'tokens_out',
		label: 'Tokens Out',
		description: 'Total output tokens',
		defaultColor: 'magenta',
		formats: [
			{ value: 'compact', label: 'Compact (12k)' },
			{ value: 'full', label: 'Full (12000)' }
		]
	},
	{
		type: 'duration',
		label: 'Duration',
		description: 'Total session wall-clock time',
		defaultColor: 'cyan',
		formats: [
			{ value: 'short', label: 'Short (5m 30s)' },
			{ value: 'hms', label: 'H:M:S (0:05:30)' }
		]
	},
	{
		type: 'api_duration',
		label: 'API Duration',
		description: 'Total time waiting for API responses',
		defaultColor: 'cyan',
		formats: [
			{ value: 'short', label: 'Short (2m 10s)' },
			{ value: 'hms', label: 'H:M:S (0:02:10)' }
		]
	},
	{
		type: 'lines_changed',
		label: 'Lines Changed',
		description: 'Lines of code added and removed',
		defaultColor: 'green',
		formats: [
			{ value: 'both', label: 'Both (+156 -23)' },
			{ value: 'net', label: 'Net (+133)' }
		]
	},
	{
		type: 'git_branch',
		label: 'Git Branch',
		description: 'Current git branch name',
		defaultColor: 'bright_green'
	},
	{
		type: 'git_status',
		label: 'Git Status',
		description: 'Staged and modified file counts',
		defaultColor: 'yellow',
		formats: [
			{ value: 'compact', label: 'Compact (+3 ~5)' },
			{ value: 'verbose', label: 'Verbose (3 staged, 5 modified)' }
		]
	},
	{
		type: 'session_id',
		label: 'Session ID',
		description: 'Unique session identifier',
		defaultColor: 'gray',
		formats: [
			{ value: 'short', label: 'Short (first 8 chars)' },
			{ value: 'full', label: 'Full ID' }
		]
	},
	{
		type: 'version',
		label: 'Version',
		description: 'Claude Code version',
		defaultColor: 'gray'
	},
	{
		type: 'agent_name',
		label: 'Agent Name',
		description: 'Agent name (when using --agent flag)',
		defaultColor: 'bright_cyan'
	},
	{
		type: 'five_hour_usage',
		label: '5-Hour Usage',
		description: '5-hour rolling window usage % (requires OAuth login)',
		defaultColor: 'cyan',
		formats: [
			{ value: 'text', label: 'Text (12% 3h20m)' },
			{ value: 'bar', label: 'Bar ([██░░░░] 12%)' },
			{ value: 'percent_only', label: 'Percent only (12%)' }
		]
	},
	{
		type: 'weekly_usage',
		label: 'Weekly Usage',
		description: '7-day rolling window usage % (requires OAuth login)',
		defaultColor: 'green',
		formats: [
			{ value: 'text', label: 'Text (45% wk 85%)' },
			{ value: 'bar', label: 'Bar ([████░░] 45%)' },
			{ value: 'percent_only', label: 'Percent only (45%)' }
		]
	},
	{
		type: 'vim_mode',
		label: 'Vim Mode',
		description: 'Current vim mode (if enabled)',
		defaultColor: 'bright_yellow'
	},
	{
		type: 'separator',
		label: 'Separator',
		description: 'Visual separator between segments',
		defaultColor: 'gray'
	},
	{
		type: 'line_break',
		label: 'Line Break',
		description: 'Start a new line in the status bar',
		defaultColor: 'gray'
	},
	{
		type: 'custom_text',
		label: 'Custom Text',
		description: 'Any custom text string',
		defaultColor: 'white'
	}
];

/** Default background colors for Powerline theme segments */
export const POWERLINE_DEFAULT_BG: Record<string, SegmentColor> = {
	model: 'blue',
	cost: 'green',
	context: 'yellow',
	context_remaining: 'green',
	cwd: 'blue',
	project_dir: 'blue',
	tokens_in: 'magenta',
	tokens_out: 'magenta',
	duration: 'cyan',
	api_duration: 'cyan',
	lines_changed: 'green',
	git_branch: 'green',
	git_status: 'yellow',
	session_id: 'gray',
	version: 'gray',
	agent_name: 'cyan',
	five_hour_usage: 'cyan',
	weekly_usage: 'green',
	vim_mode: 'yellow',
	custom_text: 'gray'
};

export const SEGMENT_COLORS: { value: SegmentColor; label: string; hex: string }[] = [
	{ value: 'red', label: 'Red', hex: '#cd3131' },
	{ value: 'green', label: 'Green', hex: '#0dbc79' },
	{ value: 'yellow', label: 'Yellow', hex: '#e5e510' },
	{ value: 'blue', label: 'Blue', hex: '#2472c8' },
	{ value: 'magenta', label: 'Magenta', hex: '#bc3fbc' },
	{ value: 'cyan', label: 'Cyan', hex: '#11a8cd' },
	{ value: 'white', label: 'White', hex: '#e5e5e5' },
	{ value: 'bright_red', label: 'Bright Red', hex: '#f14c4c' },
	{ value: 'bright_green', label: 'Bright Green', hex: '#23d18b' },
	{ value: 'bright_yellow', label: 'Bright Yellow', hex: '#f5f543' },
	{ value: 'bright_blue', label: 'Bright Blue', hex: '#3b8eea' },
	{ value: 'bright_magenta', label: 'Bright Magenta', hex: '#d670d6' },
	{ value: 'bright_cyan', label: 'Bright Cyan', hex: '#29b8db' },
	{ value: 'bright_white', label: 'Bright White', hex: '#ffffff' },
	{ value: 'gray', label: 'Gray', hex: '#808080' }
];
