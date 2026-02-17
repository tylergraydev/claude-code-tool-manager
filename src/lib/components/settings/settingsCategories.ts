import { Sliders, ShieldCheck, Puzzle, Variable, ToggleRight, FileSearch, Clock, KeyRound, ServerCog, Keyboard, RotateCw, Building, Settings } from 'lucide-svelte';

export type SettingsCategoryType = 'scoped' | 'standalone';

export interface SettingsCategory {
	id: string;
	label: string;
	icon: typeof Sliders;
	type: SettingsCategoryType;
}

export const SETTINGS_CATEGORIES: SettingsCategory[] = [
	{ id: 'models', label: 'Models', icon: Sliders, type: 'scoped' },
	{ id: 'security', label: 'Security', icon: ShieldCheck, type: 'scoped' },
	{ id: 'plugins', label: 'Plugins', icon: Puzzle, type: 'scoped' },
	{ id: 'environment', label: 'Environment', icon: Variable, type: 'scoped' },
	{ id: 'interface', label: 'Interface', icon: ToggleRight, type: 'scoped' },
	{ id: 'files', label: 'Files', icon: FileSearch, type: 'scoped' },
	{ id: 'session', label: 'Session', icon: Clock, type: 'scoped' },
	{ id: 'authentication', label: 'Auth', icon: KeyRound, type: 'scoped' },
	{ id: 'mcp-approval', label: 'MCP Approval', icon: ServerCog, type: 'scoped' },
	{ id: 'keybindings', label: 'Keybindings', icon: Keyboard, type: 'standalone' },
	{ id: 'spinner-verbs', label: 'Spinner Verbs', icon: RotateCw, type: 'standalone' },
	{ id: 'admin', label: 'Admin', icon: Building, type: 'standalone' },
	{ id: 'editor-sync', label: 'Editor Sync', icon: Settings, type: 'standalone' }
];
