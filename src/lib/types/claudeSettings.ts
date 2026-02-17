export type ClaudeSettingsScope = 'user' | 'project' | 'local';

export interface SandboxNetworkSettings {
	allowUnixSockets?: string[];
	allowAllUnixSockets?: boolean;
	allowLocalBinding?: boolean;
	allowedDomains?: string[];
	httpProxyPort?: number;
	socksProxyPort?: number;
}

export interface SandboxSettings {
	enabled?: boolean;
	autoAllowBashIfSandboxed?: boolean;
	excludedCommands?: string[];
	allowUnsandboxedCommands?: boolean;
	enableWeakerNestedSandbox?: boolean;
	network?: SandboxNetworkSettings;
}

// Plugin/Marketplace types
export type MarketplaceSourceType =
	| 'github'
	| 'git'
	| 'url'
	| 'npm'
	| 'file'
	| 'directory'
	| 'hostPattern';

export type MarketplaceSource =
	| { source: 'github'; repo: string; ref?: string; path?: string }
	| { source: 'git'; url: string; ref?: string; path?: string }
	| { source: 'url'; url: string }
	| { source: 'npm'; package: string }
	| { source: 'file'; path: string }
	| { source: 'directory'; path: string }
	| { source: 'hostPattern'; hostPattern: string };

export interface MarketplaceDefinition {
	source: MarketplaceSource;
	installLocation?: string;
}

export interface KnownEnvVar {
	key: string;
	description: string;
	category: string;
}

export interface ClaudeSettings {
	scope: string;
	model?: string;
	availableModels: string[];
	outputStyle?: string;
	language?: string;
	alwaysThinkingEnabled?: boolean;
	attributionCommit?: string;
	attributionPr?: string;
	sandbox?: SandboxSettings;
	// Plugins
	enabledPlugins?: Record<string, boolean | string[]>;
	extraKnownMarketplaces?: Record<string, MarketplaceDefinition>;
	// Environment Variables
	env?: Record<string, string>;
	// UI Toggles
	showTurnDuration?: boolean;
	spinnerTipsEnabled?: boolean;
	terminalProgressBarEnabled?: boolean;
	prefersReducedMotion?: boolean;
	respectGitignore?: boolean;
}

export interface AllClaudeSettings {
	user: ClaudeSettings;
	project?: ClaudeSettings;
	local?: ClaudeSettings;
}

export const CLAUDE_MODELS = [
	{
		value: 'claude-sonnet-4-5-20250929',
		label: 'Claude Sonnet 4.5',
		description: 'Best balance of speed and intelligence'
	},
	{
		value: 'claude-opus-4-6',
		label: 'Claude Opus 4.6',
		description: 'Most capable model for complex tasks'
	},
	{
		value: 'claude-haiku-4-5-20251001',
		label: 'Claude Haiku 4.5',
		description: 'Fastest model for simple tasks'
	}
] as const;

export const AVAILABLE_MODEL_SHORTCUTS = [
	{ value: 'sonnet', label: 'Sonnet' },
	{ value: 'opus', label: 'Opus' },
	{ value: 'haiku', label: 'Haiku' }
] as const;

export const OUTPUT_STYLES = [
	{ value: '', label: 'Not set' },
	{ value: 'concise', label: 'Concise' },
	{ value: 'verbose', label: 'Verbose' },
	{ value: 'markdown', label: 'Markdown' }
] as const;

export const COMMON_LANGUAGES = [
	{ value: '', label: 'Not set (English default)' },
	{ value: 'english', label: 'English' },
	{ value: 'spanish', label: 'Spanish' },
	{ value: 'french', label: 'French' },
	{ value: 'german', label: 'German' },
	{ value: 'italian', label: 'Italian' },
	{ value: 'portuguese', label: 'Portuguese' },
	{ value: 'japanese', label: 'Japanese' },
	{ value: 'korean', label: 'Korean' },
	{ value: 'chinese', label: 'Chinese' },
	{ value: 'russian', label: 'Russian' },
	{ value: 'arabic', label: 'Arabic' },
	{ value: 'hindi', label: 'Hindi' },
	{ value: 'dutch', label: 'Dutch' },
	{ value: 'swedish', label: 'Swedish' },
	{ value: 'polish', label: 'Polish' },
	{ value: 'turkish', label: 'Turkish' },
	{ value: 'thai', label: 'Thai' },
	{ value: 'vietnamese', label: 'Vietnamese' }
] as const;

export const CLAUDE_SETTINGS_SCOPE_LABELS: Record<
	ClaudeSettingsScope,
	{ label: string; description: string }
> = {
	user: {
		label: 'User',
		description: '~/.claude/settings.json — applies to all projects'
	},
	project: {
		label: 'Project',
		description: '.claude/settings.json — shared with team via git'
	},
	local: {
		label: 'Local',
		description: '.claude/settings.local.json — local overrides, not committed'
	}
};

export const UI_TOGGLE_FIELDS = [
	{
		key: 'showTurnDuration' as const,
		label: 'Show Turn Duration',
		description: 'Display how long each turn takes in the conversation',
		defaultValue: false
	},
	{
		key: 'spinnerTipsEnabled' as const,
		label: 'Spinner Tips',
		description: 'Show helpful tips in the loading spinner',
		defaultValue: true
	},
	{
		key: 'terminalProgressBarEnabled' as const,
		label: 'Terminal Progress Bar',
		description: 'Show a progress bar in the terminal during operations',
		defaultValue: true
	},
	{
		key: 'prefersReducedMotion' as const,
		label: 'Reduced Motion',
		description: 'Minimize animations and motion effects',
		defaultValue: false
	},
	{
		key: 'respectGitignore' as const,
		label: 'Respect .gitignore',
		description: 'Honor .gitignore rules when searching and listing files',
		defaultValue: true
	}
] as const;

export const KNOWN_ENV_VARS: KnownEnvVar[] = [
	// API & Authentication
	{ key: 'ANTHROPIC_API_KEY', description: 'Anthropic API key for Claude', category: 'API & Auth' },
	{ key: 'ANTHROPIC_AUTH_TOKEN', description: 'OAuth/authentication token', category: 'API & Auth' },
	{ key: 'CLAUDE_CODE_API_KEY', description: 'API key specifically for Claude Code', category: 'API & Auth' },
	// Model & Provider
	{ key: 'ANTHROPIC_MODEL', description: 'Override the default model', category: 'Model & Provider' },
	{ key: 'ANTHROPIC_SMALL_FAST_MODEL', description: 'Model for fast/cheap operations', category: 'Model & Provider' },
	{ key: 'CLAUDE_CODE_MAX_MODEL', description: 'Maximum model tier to use', category: 'Model & Provider' },
	{ key: 'CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC', description: 'Disable telemetry and non-essential API calls', category: 'Model & Provider' },
	// Network & Proxy
	{ key: 'ANTHROPIC_BASE_URL', description: 'Custom API base URL', category: 'Network & Proxy' },
	{ key: 'HTTP_PROXY', description: 'HTTP proxy URL', category: 'Network & Proxy' },
	{ key: 'HTTPS_PROXY', description: 'HTTPS proxy URL', category: 'Network & Proxy' },
	{ key: 'NO_PROXY', description: 'Comma-separated list of hosts to bypass proxy', category: 'Network & Proxy' },
	{ key: 'CLAUDE_CODE_API_TIMEOUT', description: 'API request timeout in milliseconds', category: 'Network & Proxy' },
	// Behavior & Limits
	{ key: 'CLAUDE_CODE_MAX_TURNS', description: 'Maximum number of agentic turns', category: 'Behavior & Limits' },
	{ key: 'CLAUDE_CODE_MAX_OUTPUT_TOKENS', description: 'Maximum output tokens per response', category: 'Behavior & Limits' },
	{ key: 'CLAUDE_CODE_BUDGET_TOKENS', description: 'Token budget for extended thinking', category: 'Behavior & Limits' },
	{ key: 'CLAUDE_CODE_USE_BEDROCK', description: 'Use AWS Bedrock as provider (1 to enable)', category: 'Behavior & Limits' },
	{ key: 'CLAUDE_CODE_USE_VERTEX', description: 'Use Google Vertex AI as provider (1 to enable)', category: 'Behavior & Limits' },
	// AWS Bedrock
	{ key: 'AWS_REGION', description: 'AWS region for Bedrock', category: 'AWS Bedrock' },
	{ key: 'AWS_ACCESS_KEY_ID', description: 'AWS access key ID', category: 'AWS Bedrock' },
	{ key: 'AWS_SECRET_ACCESS_KEY', description: 'AWS secret access key', category: 'AWS Bedrock' },
	{ key: 'AWS_SESSION_TOKEN', description: 'AWS session token', category: 'AWS Bedrock' },
	{ key: 'AWS_PROFILE', description: 'AWS CLI profile name', category: 'AWS Bedrock' },
	{ key: 'ANTHROPIC_BEDROCK_BASE_URL', description: 'Custom Bedrock endpoint URL', category: 'AWS Bedrock' },
	// Google Vertex
	{ key: 'CLOUD_ML_REGION', description: 'Google Cloud region for Vertex AI', category: 'Google Vertex' },
	{ key: 'ANTHROPIC_VERTEX_PROJECT_ID', description: 'Google Cloud project ID', category: 'Google Vertex' },
	{ key: 'ANTHROPIC_VERTEX_BASE_URL', description: 'Custom Vertex AI endpoint URL', category: 'Google Vertex' },
	// Display & UI
	{ key: 'CLAUDE_CODE_OUTPUT_FORMAT', description: 'Output format (text, json, stream-json)', category: 'Display & UI' },
	{ key: 'CLAUDE_CODE_THEME', description: 'Color theme override', category: 'Display & UI' },
	{ key: 'CLAUDE_CODE_TERMINAL_EMULATOR', description: 'Terminal emulator type hint', category: 'Display & UI' },
	// Sandbox & Security
	{ key: 'CLAUDE_CODE_SANDBOX_ENABLED', description: 'Enable sandbox mode (1 to enable)', category: 'Sandbox & Security' },
	{ key: 'CLAUDE_CODE_SANDBOX_DEBUG', description: 'Enable sandbox debug logging', category: 'Sandbox & Security' },
	// Git & VCS
	{ key: 'CLAUDE_CODE_GIT_AUTHOR_NAME', description: 'Override git author name for commits', category: 'Git & VCS' },
	{ key: 'CLAUDE_CODE_GIT_AUTHOR_EMAIL', description: 'Override git author email for commits', category: 'Git & VCS' },
	// MCP
	{ key: 'CLAUDE_CODE_MCP_TIMEOUT', description: 'Timeout for MCP server connections (ms)', category: 'MCP' },
	{ key: 'CLAUDE_CODE_MCP_AUTO_START', description: 'Auto-start MCP servers on launch', category: 'MCP' },
	// Logging & Debug
	{ key: 'CLAUDE_CODE_DEBUG', description: 'Enable debug mode (1 to enable)', category: 'Logging & Debug' },
	{ key: 'CLAUDE_CODE_LOG_LEVEL', description: 'Log level (debug, info, warn, error)', category: 'Logging & Debug' },
	{ key: 'CLAUDE_CODE_LOG_FILE', description: 'Path to log file', category: 'Logging & Debug' },
	// System
	{ key: 'CLAUDE_CODE_SKIP_UPDATE_CHECK', description: 'Skip checking for updates', category: 'System' },
	{ key: 'CLAUDE_CODE_CONFIG_DIR', description: 'Override config directory path', category: 'System' },
	{ key: 'CLAUDE_CODE_CACHE_DIR', description: 'Override cache directory path', category: 'System' },
	{ key: 'TMPDIR', description: 'Temporary directory for Claude operations', category: 'System' }
];

export const ENV_VAR_CATEGORIES = [
	...new Set(KNOWN_ENV_VARS.map((v) => v.category))
] as const;

export const MARKETPLACE_SOURCE_TYPES: { value: MarketplaceSourceType; label: string }[] = [
	{ value: 'github', label: 'GitHub Repository' },
	{ value: 'git', label: 'Git URL' },
	{ value: 'url', label: 'URL' },
	{ value: 'npm', label: 'NPM Package' },
	{ value: 'file', label: 'File' },
	{ value: 'directory', label: 'Directory' },
	{ value: 'hostPattern', label: 'Host Pattern' }
];
