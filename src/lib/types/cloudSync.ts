export interface SyncAuthStatus {
	isAuthenticated: boolean;
	username: string | null;
	hasGhCli: boolean;
	gistId: string | null;
	gistUrl: string | null;
}

export interface SyncConfig {
	syncGlobalClaudeMd: boolean;
	syncSkills: boolean;
	syncMcps: boolean;
	syncProjectClaudeMds: string[];
	autoSyncOnLaunch: boolean;
}

export interface ProjectMapping {
	localPath: string;
	canonicalName: string;
}

export interface SyncStatus {
	lastPushedAt: string | null;
	lastPulledAt: string | null;
	gistId: string | null;
	gistUrl: string | null;
	itemCounts: SyncItemCounts;
}

export interface SyncItemCounts {
	mcps: number;
	skills: number;
	projects: number;
	hasGlobalClaudeMd: boolean;
}

export interface CloudSyncResult {
	pushed: string[];
	pulled: string[];
	conflicts: string[];
	syncedAt: string;
	gistUrl: string;
}
