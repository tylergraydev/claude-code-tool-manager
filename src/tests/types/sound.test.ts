import { describe, it, expect } from 'vitest';
import {
	getDefaultSound,
	getSuggestedSound,
	SOUND_HOOK_PRESETS,
	MACOS_SYSTEM_SOUNDS
} from '$lib/types/sound';

describe('Sound Types', () => {
	describe('getDefaultSound', () => {
		it('should return the default Glass sound path', () => {
			const defaultSound = getDefaultSound();
			expect(defaultSound).toBe('/System/Library/Sounds/Glass.aiff');
		});
	});

	describe('getSuggestedSound', () => {
		it('should suggest Glass for Stop event', () => {
			expect(getSuggestedSound('Stop')).toBe('/System/Library/Sounds/Glass.aiff');
		});

		it('should suggest Glass for SubagentStop event', () => {
			expect(getSuggestedSound('SubagentStop')).toBe('/System/Library/Sounds/Glass.aiff');
		});

		it('should suggest Ping for Notification event', () => {
			expect(getSuggestedSound('Notification')).toBe('/System/Library/Sounds/Ping.aiff');
		});

		it('should suggest Hero for SessionStart event', () => {
			expect(getSuggestedSound('SessionStart')).toBe('/System/Library/Sounds/Hero.aiff');
		});

		it('should suggest Blow for SessionEnd event', () => {
			expect(getSuggestedSound('SessionEnd')).toBe('/System/Library/Sounds/Blow.aiff');
		});

		it('should suggest Pop for other events', () => {
			expect(getSuggestedSound('PreToolUse')).toBe('/System/Library/Sounds/Pop.aiff');
			expect(getSuggestedSound('PostToolUse')).toBe('/System/Library/Sounds/Pop.aiff');
			expect(getSuggestedSound('UserPromptSubmit')).toBe('/System/Library/Sounds/Pop.aiff');
		});
	});

	describe('SOUND_HOOK_PRESETS', () => {
		it('should have task-complete preset', () => {
			const preset = SOUND_HOOK_PRESETS.find((p) => p.id === 'task-complete');
			expect(preset).toBeDefined();
			expect(preset?.name).toBe('Task Complete');
			expect(preset?.events).toContain('Stop');
			expect(preset?.events).toContain('SubagentStop');
		});

		it('should have permission-required preset', () => {
			const preset = SOUND_HOOK_PRESETS.find((p) => p.id === 'permission-required');
			expect(preset).toBeDefined();
			expect(preset?.name).toBe('Permission Required');
			expect(preset?.events).toContain('Notification');
		});

		it('should have full-suite preset', () => {
			const preset = SOUND_HOOK_PRESETS.find((p) => p.id === 'full-suite');
			expect(preset).toBeDefined();
			expect(preset?.name).toBe('Full Notification Suite');
			expect(preset?.events).toContain('Stop');
			expect(preset?.events).toContain('SubagentStop');
			expect(preset?.events).toContain('Notification');
		});

		it('should have required fields on all presets', () => {
			for (const preset of SOUND_HOOK_PRESETS) {
				expect(preset.id).toBeDefined();
				expect(preset.name).toBeDefined();
				expect(preset.description).toBeDefined();
				expect(preset.events).toBeDefined();
				expect(preset.events.length).toBeGreaterThan(0);
				expect(preset.icon).toBeDefined();
			}
		});
	});

	describe('MACOS_SYSTEM_SOUNDS', () => {
		it('should contain common macOS sounds', () => {
			expect(MACOS_SYSTEM_SOUNDS).toContain('Glass');
			expect(MACOS_SYSTEM_SOUNDS).toContain('Ping');
			expect(MACOS_SYSTEM_SOUNDS).toContain('Pop');
			expect(MACOS_SYSTEM_SOUNDS).toContain('Sosumi');
		});

		it('should have expected number of sounds', () => {
			expect(MACOS_SYSTEM_SOUNDS.length).toBe(14);
		});
	});
});
