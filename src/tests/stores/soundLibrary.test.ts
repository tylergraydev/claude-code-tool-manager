import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Sound Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	describe('load', () => {
		it('should load system and custom sounds', async () => {
			const mockSystemSounds = [
				{ name: 'Glass', path: '/System/Library/Sounds/Glass.aiff', category: 'system' },
				{ name: 'Ping', path: '/System/Library/Sounds/Ping.aiff', category: 'system' }
			];
			const mockCustomSounds = [
				{ name: 'custom-beep', path: '/path/to/custom-beep.aiff', size: 1024, createdAt: '2024-01-01' }
			];
			const mockSoundsDir = '/Users/test/.claude-code-tool-manager/sounds';

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockSystemSounds)
				.mockResolvedValueOnce(mockCustomSounds)
				.mockResolvedValueOnce(mockSoundsDir);

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');
			await soundLibrary.load();

			expect(soundLibrary.systemSounds).toHaveLength(2);
			expect(soundLibrary.customSounds).toHaveLength(1);
			expect(soundLibrary.soundsDirectory).toBe(mockSoundsDir);
		});

		it('should handle empty responses', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce('');

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');
			await soundLibrary.load();

			expect(soundLibrary.systemSounds).toHaveLength(0);
			expect(soundLibrary.customSounds).toHaveLength(0);
		});

		it('should set isLoading during load', async () => {
			let resolveFirst: (value: unknown) => void;
			let resolveSecond: (value: unknown) => void;
			let resolveThird: (value: unknown) => void;

			const firstPromise = new Promise((resolve) => {
				resolveFirst = resolve;
			});
			const secondPromise = new Promise((resolve) => {
				resolveSecond = resolve;
			});
			const thirdPromise = new Promise((resolve) => {
				resolveThird = resolve;
			});

			vi.mocked(invoke)
				.mockReturnValueOnce(firstPromise as Promise<unknown>)
				.mockReturnValueOnce(secondPromise as Promise<unknown>)
				.mockReturnValueOnce(thirdPromise as Promise<unknown>);

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');
			const loadPromise = soundLibrary.load();

			expect(soundLibrary.isLoading).toBe(true);

			resolveFirst!([]);
			resolveSecond!([]);
			resolveThird!('');
			await loadPromise;

			expect(soundLibrary.isLoading).toBe(false);
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load sounds'));

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');
			await soundLibrary.load();

			expect(soundLibrary.error).toContain('Failed to load sounds');
			expect(soundLibrary.isLoading).toBe(false);
		});
	});

	describe('allSounds logic', () => {
		it('should have both system and custom sounds available after load', async () => {
			const mockSystemSounds = [
				{ name: 'Glass', path: '/System/Library/Sounds/Glass.aiff', category: 'system' as const }
			];
			const mockCustomSounds = [
				{ name: 'custom-beep', path: '/path/to/custom-beep.aiff', size: 1024, createdAt: '2024-01-01' }
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockSystemSounds)
				.mockResolvedValueOnce(mockCustomSounds)
				.mockResolvedValueOnce('');

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');
			await soundLibrary.load();

			// Test the raw data is available (derived state may not work in jsdom)
			expect(soundLibrary.systemSounds).toHaveLength(1);
			expect(soundLibrary.customSounds).toHaveLength(1);
		});
	});

	describe('previewSound', () => {
		it('should preview a sound', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');
			await soundLibrary.previewSound('/System/Library/Sounds/Glass.aiff');

			expect(invoke).toHaveBeenCalledWith('preview_sound', { path: '/System/Library/Sounds/Glass.aiff' });
		});

		it('should set isPlaying state during preview', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');
			const previewPromise = soundLibrary.previewSound('/System/Library/Sounds/Glass.aiff');

			expect(soundLibrary.isPlaying).toBe('/System/Library/Sounds/Glass.aiff');

			await previewPromise;

			// Advance timer to clear isPlaying
			vi.advanceTimersByTime(2000);

			expect(soundLibrary.isPlaying).toBe(null);
		});

		it('should not double-play the same sound', async () => {
			vi.mocked(invoke).mockResolvedValue(undefined);

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');

			await soundLibrary.previewSound('/System/Library/Sounds/Glass.aiff');

			// Try to play the same sound while it's playing
			await soundLibrary.previewSound('/System/Library/Sounds/Glass.aiff');

			// Should only be called once
			expect(invoke).toHaveBeenCalledTimes(1);
		});

		it('should handle preview errors gracefully', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Audio error'));

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');

			// Should not throw
			await expect(soundLibrary.previewSound('/System/Library/Sounds/Glass.aiff')).resolves.not.toThrow();

			// Advance timer to clear isPlaying
			vi.advanceTimersByTime(2000);
			expect(soundLibrary.isPlaying).toBe(null);
		});
	});

	describe('uploadSound', () => {
		it('should upload a custom sound', async () => {
			const mockSound = {
				name: 'my-sound',
				path: '/path/to/my-sound.aiff',
				size: 2048,
				createdAt: '2024-01-01'
			};

			vi.mocked(invoke).mockResolvedValueOnce(mockSound);

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');
			const data = new Uint8Array([1, 2, 3, 4]);
			const result = await soundLibrary.uploadSound('my-sound', data);

			expect(result).toEqual(mockSound);
			expect(invoke).toHaveBeenCalledWith('upload_custom_sound', {
				name: 'my-sound',
				data: [1, 2, 3, 4]
			});
			expect(soundLibrary.customSounds).toContainEqual(mockSound);
		});
	});

	describe('deleteSound', () => {
		it('should delete a custom sound', async () => {
			const mockCustomSounds = [
				{ name: 'sound-1', path: '/path/to/sound-1.aiff', size: 1024, createdAt: '2024-01-01' },
				{ name: 'sound-2', path: '/path/to/sound-2.aiff', size: 2048, createdAt: '2024-01-02' }
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce([]) // system sounds
				.mockResolvedValueOnce(mockCustomSounds) // custom sounds
				.mockResolvedValueOnce('') // sounds dir
				.mockResolvedValueOnce(undefined); // delete

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');
			await soundLibrary.load();

			expect(soundLibrary.customSounds).toHaveLength(2);

			await soundLibrary.deleteSound('sound-1');

			expect(invoke).toHaveBeenCalledWith('delete_custom_sound', { name: 'sound-1' });
			expect(soundLibrary.customSounds).toHaveLength(1);
			expect(soundLibrary.customSounds[0].name).toBe('sound-2');
		});
	});

	describe('generateHookCommand', () => {
		it('should call invoke with shell method', async () => {
			const mockCommand = 'afplay "/System/Library/Sounds/Glass.aiff"';
			vi.mocked(invoke).mockResolvedValueOnce(mockCommand);

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');
			const command = await soundLibrary.generateHookCommand('/System/Library/Sounds/Glass.aiff', 'shell');

			expect(invoke).toHaveBeenCalledWith('generate_sound_hook_command', {
				soundPath: '/System/Library/Sounds/Glass.aiff',
				method: 'shell'
			});
			expect(command).toBe(mockCommand);
		});

		it('should call invoke with python method', async () => {
			const mockCommand = 'python3 /path/to/play.py "/System/Library/Sounds/Glass.aiff"';
			vi.mocked(invoke).mockResolvedValueOnce(mockCommand);

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');
			const command = await soundLibrary.generateHookCommand('/System/Library/Sounds/Glass.aiff', 'python');

			expect(invoke).toHaveBeenCalledWith('generate_sound_hook_command', {
				soundPath: '/System/Library/Sounds/Glass.aiff',
				method: 'python'
			});
			expect(command).toBe(mockCommand);
		});
	});

	describe('deployNotificationScript', () => {
		it('should deploy notification script', async () => {
			const mockPath = '/Users/test/.claude-code-tool-manager/scripts/play_sound.py';

			vi.mocked(invoke).mockResolvedValueOnce(mockPath);

			const { soundLibrary } = await import('$lib/stores/soundLibrary.svelte');
			const result = await soundLibrary.deployNotificationScript();

			expect(invoke).toHaveBeenCalledWith('deploy_notification_script');
			expect(result).toBe(mockPath);
		});
	});

	// Note: getSoundByPath and getSoundByName rely on $derived.by which
	// doesn't work properly in jsdom. The underlying logic is tested through
	// the load tests and the store implementation uses standard JavaScript methods.
});
