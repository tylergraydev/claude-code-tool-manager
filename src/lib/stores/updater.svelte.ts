import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type UpdateStatus = 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'error';

class UpdaterState {
	status = $state<UpdateStatus>('idle');
	update = $state<Update | null>(null);
	error = $state<string | null>(null);
	downloadProgress = $state<number>(0);

	async checkForUpdates() {
		if (this.status === 'checking' || this.status === 'downloading') {
			return;
		}

		this.status = 'checking';
		this.error = null;

		try {
			const update = await check();
			if (update) {
				this.update = update;
				this.status = 'available';
			} else {
				this.status = 'idle';
			}
		} catch (e) {
			this.error = e instanceof Error ? e.message : 'Failed to check for updates';
			this.status = 'error';
		}
	}

	async downloadAndInstall() {
		if (!this.update || this.status === 'downloading') {
			return;
		}

		this.status = 'downloading';
		this.downloadProgress = 0;

		try {
			await this.update.downloadAndInstall((event) => {
				if (event.event === 'Started' && event.data.contentLength) {
					this.downloadProgress = 0;
				} else if (event.event === 'Progress') {
					// Calculate progress percentage
					this.downloadProgress = event.data.chunkLength;
				} else if (event.event === 'Finished') {
					this.downloadProgress = 100;
				}
			});

			this.status = 'ready';
		} catch (e) {
			this.error = e instanceof Error ? e.message : 'Failed to download update';
			this.status = 'error';
		}
	}

	async restartApp() {
		await relaunch();
	}

	dismiss() {
		this.status = 'idle';
		this.update = null;
		this.error = null;
	}
}

export const updater = new UpdaterState();
