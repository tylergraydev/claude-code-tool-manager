import type { HookEventType } from './hook';

/** Represents a sound file that can be played */
export interface SystemSound {
	name: string;
	path: string;
	category: 'system' | 'custom';
}

/** Represents a custom uploaded sound */
export interface CustomSound {
	name: string;
	path: string;
	size: number;
	createdAt: string;
}

/** Sound playback method */
export type SoundMethod = 'shell' | 'python';

/** Configuration for a sound hook */
export interface SoundHookConfig {
	eventType: HookEventType;
	soundPath: string;
	method: SoundMethod;
	enabled: boolean;
}

/** Preset for quick sound hook setup */
export interface SoundHookPreset {
	id: string;
	name: string;
	description: string;
	events: HookEventType[];
	icon: string;
}

/** Predefined sound hook presets */
export const SOUND_HOOK_PRESETS: SoundHookPreset[] = [
	{
		id: 'task-complete',
		name: 'Task Complete',
		description: 'Play sound when Claude finishes responding',
		events: ['Stop', 'SubagentStop'],
		icon: 'check-circle'
	},
	{
		id: 'permission-required',
		name: 'Permission Required',
		description: 'Alert when Claude needs permission',
		events: ['Notification'],
		icon: 'shield-alert'
	},
	{
		id: 'full-suite',
		name: 'Full Notification Suite',
		description: 'Sounds for all key events',
		events: ['Stop', 'SubagentStop', 'Notification'],
		icon: 'bell-ring'
	}
];

/** Common macOS system sounds */
export const MACOS_SYSTEM_SOUNDS = [
	'Basso',
	'Blow',
	'Bottle',
	'Frog',
	'Funk',
	'Glass',
	'Hero',
	'Morse',
	'Ping',
	'Pop',
	'Purr',
	'Sosumi',
	'Submarine',
	'Tink'
];

/** Get the default sound based on platform */
export function getDefaultSound(): string {
	// Default to Glass on macOS (most pleasant notification sound)
	return '/System/Library/Sounds/Glass.aiff';
}

/** Get a suggested sound for a given event type */
export function getSuggestedSound(eventType: HookEventType): string {
	switch (eventType) {
		case 'Stop':
		case 'SubagentStop':
			return '/System/Library/Sounds/Glass.aiff';
		case 'Notification':
			return '/System/Library/Sounds/Ping.aiff';
		case 'SessionStart':
			return '/System/Library/Sounds/Hero.aiff';
		case 'SessionEnd':
			return '/System/Library/Sounds/Blow.aiff';
		default:
			return '/System/Library/Sounds/Pop.aiff';
	}
}
