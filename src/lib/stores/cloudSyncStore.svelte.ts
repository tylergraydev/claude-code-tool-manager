import { invoke } from '@tauri-apps/api/core';
import type {
	SyncAuthStatus,
	SyncConfig,
	SyncStatus,
	CloudSyncResult,
	ProjectMapping
} from '$lib/types';

class CloudSyncState {
	authStatus = $state<SyncAuthStatus | null>(null);
	syncConfig = $state<SyncConfig | null>(null);
	syncStatus = $state<SyncStatus | null>(null);
	projectMappings = $state<ProjectMapping[]>([]);
	isLoading = $state(false);
	isPushing = $state(false);
	isPulling = $state(false);
	isConnecting = $state(false);
	error = $state<string | null>(null);
	lastResult = $state<CloudSyncResult | null>(null);

	isAuthenticated = $derived(this.authStatus?.isAuthenticated ?? false);

	async load() {
		console.log('[cloudSync] Loading cloud sync state...');
		this.isLoading = true;
		this.error = null;
		try {
			const [auth, config, status, mappings] = await Promise.all([
				invoke<SyncAuthStatus>('get_sync_auth_status'),
				invoke<SyncConfig>('get_sync_config'),
				invoke<SyncStatus>('get_sync_status'),
				invoke<ProjectMapping[]>('get_project_mappings')
			]);
			this.authStatus = auth;
			this.syncConfig = config;
			this.syncStatus = status;
			this.projectMappings = mappings;
			console.log('[cloudSync] Loaded:', { auth: auth.isAuthenticated, username: auth.username });
		} catch (e) {
			this.error = String(e);
			console.error('[cloudSync] Failed to load:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async connect() {
		console.log('[cloudSync] Connecting via gh CLI...');
		this.isConnecting = true;
		this.error = null;
		try {
			this.authStatus = await invoke<SyncAuthStatus>('connect_cloud_sync');
			// Reload status after connecting
			this.syncStatus = await invoke<SyncStatus>('get_sync_status');
			console.log('[cloudSync] Connected as', this.authStatus.username);
			return this.authStatus;
		} catch (e) {
			this.error = String(e);
			console.error('[cloudSync] Connection failed:', e);
			throw e;
		} finally {
			this.isConnecting = false;
		}
	}

	async disconnect() {
		console.log('[cloudSync] Disconnecting...');
		try {
			await invoke('disconnect_cloud_sync');
			this.authStatus = {
				isAuthenticated: false,
				username: null,
				hasGhCli: this.authStatus?.hasGhCli ?? false,
				gistId: null,
				gistUrl: null
			};
			this.syncStatus = null;
			this.lastResult = null;
			console.log('[cloudSync] Disconnected');
		} catch (e) {
			this.error = String(e);
			console.error('[cloudSync] Disconnect failed:', e);
		}
	}

	async saveConfig(config: SyncConfig) {
		console.log('[cloudSync] Saving sync config');
		try {
			await invoke('save_sync_config', { config });
			this.syncConfig = config;
			// Refresh status counts
			this.syncStatus = await invoke<SyncStatus>('get_sync_status');
		} catch (e) {
			this.error = String(e);
			console.error('[cloudSync] Failed to save config:', e);
		}
	}

	async saveMappings(mappings: ProjectMapping[]) {
		console.log('[cloudSync] Saving project mappings');
		try {
			await invoke('save_project_mappings', { mappings });
			this.projectMappings = mappings;
		} catch (e) {
			this.error = String(e);
			console.error('[cloudSync] Failed to save mappings:', e);
		}
	}

	async push(): Promise<CloudSyncResult | null> {
		console.log('[cloudSync] Pushing...');
		this.isPushing = true;
		this.error = null;
		try {
			const result = await invoke<CloudSyncResult>('push_sync');
			this.lastResult = result;
			// Refresh status
			this.syncStatus = await invoke<SyncStatus>('get_sync_status');
			console.log('[cloudSync] Push complete:', result.pushed);
			return result;
		} catch (e) {
			this.error = String(e);
			console.error('[cloudSync] Push failed:', e);
			return null;
		} finally {
			this.isPushing = false;
		}
	}

	async pull(): Promise<CloudSyncResult | null> {
		console.log('[cloudSync] Pulling...');
		this.isPulling = true;
		this.error = null;
		try {
			const result = await invoke<CloudSyncResult>('pull_sync');
			this.lastResult = result;
			// Refresh status
			this.syncStatus = await invoke<SyncStatus>('get_sync_status');
			console.log('[cloudSync] Pull complete:', result.pulled);
			return result;
		} catch (e) {
			this.error = String(e);
			console.error('[cloudSync] Pull failed:', e);
			return null;
		} finally {
			this.isPulling = false;
		}
	}
}

export const cloudSyncStore = new CloudSyncState();
