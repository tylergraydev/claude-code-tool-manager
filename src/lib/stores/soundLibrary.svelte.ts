import { invoke } from '@tauri-apps/api/core';
import type { SystemSound, CustomSound } from '$lib/types';

class SoundLibraryState {
	systemSounds = $state<SystemSound[]>([]);
	customSounds = $state<CustomSound[]>([]);
	soundsDirectory = $state<string>('');
	isLoading = $state(false);
	isPlaying = $state<string | null>(null);
	error = $state<string | null>(null);

	// All sounds combined (system + custom)
	allSounds = $derived.by(() => {
		const custom: SystemSound[] = this.customSounds.map((s) => ({
			name: s.name,
			path: s.path,
			category: 'custom' as const
		}));
		return [...this.systemSounds, ...custom];
	});

	async load() {
		console.log('[soundLibrary] Loading sounds...');
		this.isLoading = true;
		this.error = null;
		try {
			const [system, custom, dir] = await Promise.all([
				invoke<SystemSound[]>('get_system_sounds'),
				invoke<CustomSound[]>('get_custom_sounds'),
				invoke<string>('ensure_sounds_directory')
			]);
			this.systemSounds = system;
			this.customSounds = custom;
			this.soundsDirectory = dir;
			console.log(
				`[soundLibrary] Loaded ${system.length} system sounds and ${custom.length} custom sounds`
			);
		} catch (e) {
			this.error = String(e);
			console.error('[soundLibrary] Failed to load sounds:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async previewSound(path: string) {
		if (this.isPlaying === path) return;

		console.log(`[soundLibrary] Previewing sound: ${path}`);
		this.isPlaying = path;

		try {
			await invoke('preview_sound', { path });
		} catch (e) {
			console.error('[soundLibrary] Failed to play sound:', e);
		} finally {
			// Reset after estimated playback time (most sounds are 1-2 seconds)
			setTimeout(() => {
				if (this.isPlaying === path) {
					this.isPlaying = null;
				}
			}, 2000);
		}
	}

	async uploadSound(name: string, data: Uint8Array): Promise<CustomSound> {
		console.log(`[soundLibrary] Uploading sound: ${name}`);
		const sound = await invoke<CustomSound>('upload_custom_sound', {
			name,
			data: Array.from(data)
		});
		this.customSounds = [...this.customSounds, sound];
		console.log(`[soundLibrary] Uploaded sound: ${sound.path}`);
		return sound;
	}

	async deleteSound(name: string): Promise<void> {
		console.log(`[soundLibrary] Deleting sound: ${name}`);
		await invoke('delete_custom_sound', { name });
		this.customSounds = this.customSounds.filter((s) => s.name !== name);
		console.log(`[soundLibrary] Deleted sound: ${name}`);
	}

	async generateHookCommand(soundPath: string, method: 'shell' | 'python'): Promise<string> {
		return await invoke<string>('generate_sound_hook_command', { soundPath, method });
	}

	async deployNotificationScript(): Promise<string> {
		console.log('[soundLibrary] Deploying notification script...');
		const path = await invoke<string>('deploy_notification_script');
		console.log(`[soundLibrary] Deployed notification script to: ${path}`);
		return path;
	}

	getSoundByPath(path: string): SystemSound | undefined {
		return this.allSounds.find((s) => s.path === path);
	}

	getSoundByName(name: string): SystemSound | undefined {
		return this.allSounds.find((s) => s.name.toLowerCase() === name.toLowerCase());
	}
}

export const soundLibrary = new SoundLibraryState();
