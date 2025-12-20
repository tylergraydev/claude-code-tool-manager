export type RepoType = 'file_based' | 'readme_based';
export type ContentType = 'mcp' | 'skill' | 'subagent' | 'mixed';
export type ItemType = 'mcp' | 'skill' | 'subagent';

export interface Repo {
	id: number;
	name: string;
	owner: string;
	repo: string;
	repoType: RepoType;
	contentType: ContentType;
	githubUrl: string;
	description?: string;
	isDefault: boolean;
	isEnabled: boolean;
	lastFetchedAt?: string;
	etag?: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateRepoRequest {
	githubUrl: string;
	repoType: RepoType;
	contentType: ContentType;
}

export interface RepoItem {
	id: number;
	repoId: number;
	itemType: ItemType;
	name: string;
	description?: string;
	sourceUrl?: string;
	rawContent?: string;
	filePath?: string;
	metadata?: string;
	stars?: number;
	isImported: boolean;
	importedItemId?: number;
	createdAt: string;
	updatedAt: string;
}

export interface SyncResult {
	added: number;
	updated: number;
	removed: number;
	errors: string[];
}

export interface RateLimitInfo {
	limit: number;
	remaining: number;
	resetAt: string;
}

export interface ImportResult {
	success: boolean;
	itemType: string;
	itemId: number;
	message?: string;
}
