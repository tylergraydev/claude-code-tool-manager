import type { Mcp, CreateMcpRequest } from '$lib/types/mcp';
import type { Project, ProjectMcp } from '$lib/types/project';
import type { Skill } from '$lib/types/skill';
import type { Hook } from '$lib/types/hook';
import type { SubAgent } from '$lib/types/subagent';
import type { Command } from '$lib/types/command';
import type { Profile, ProfileWithItems } from '$lib/types/profile';
import type { StatusLine, StatusLineGalleryEntry } from '$lib/types/statusline';
import type { Repo, RepoItem, ImportResult, SyncResult, RegistryMcpEntry, RegistrySearchResult } from '$lib/types/repo';

let _nextId = 1;
function nextId(): number {
	return _nextId++;
}

export function resetIdCounter(): void {
	_nextId = 1;
}

const now = '2024-01-01T00:00:00Z';

export function createMockMcp(overrides: Partial<Mcp> = {}): Mcp {
	const id = overrides.id ?? nextId();
	return {
		id,
		name: `mcp-${id}`,
		description: null,
		type: 'stdio',
		command: 'npx',
		args: ['-y', '@example/server'],
		url: null,
		headers: null,
		env: null,
		icon: null,
		tags: null,
		source: 'manual',
		sourcePath: null,
		isEnabledGlobal: false,
		isFavorite: false,
		createdAt: now,
		updatedAt: now,
		...overrides
	};
}

export function createMockProject(overrides: Partial<Project> = {}): Project {
	const id = overrides.id ?? nextId();
	return {
		id,
		name: `project-${id}`,
		path: `/Users/test/projects/project-${id}`,
		hasMcpFile: false,
		hasSettingsFile: false,
		lastScannedAt: null,
		editorType: 'claude_code',
		isFavorite: false,
		createdAt: now,
		updatedAt: now,
		assignedMcps: [],
		...overrides
	};
}

export function createMockSkill(overrides: Partial<Skill> = {}): Skill {
	const id = overrides.id ?? nextId();
	return {
		id,
		name: `skill-${id}`,
		description: `Skill ${id} description`,
		content: 'Skill content',
		disableModelInvocation: false,
		source: 'manual',
		isFavorite: false,
		createdAt: now,
		updatedAt: now,
		...overrides
	};
}

export function createMockHook(overrides: Partial<Hook> = {}): Hook {
	const id = overrides.id ?? nextId();
	return {
		id,
		name: `hook-${id}`,
		description: `Hook ${id} description`,
		eventType: 'PreToolUse',
		hookType: 'command',
		command: 'echo test',
		source: 'manual',
		isTemplate: false,
		createdAt: now,
		updatedAt: now,
		...overrides
	};
}

export function createMockSubAgent(overrides: Partial<SubAgent> = {}): SubAgent {
	const id = overrides.id ?? nextId();
	return {
		id,
		name: `subagent-${id}`,
		description: `SubAgent ${id} description`,
		content: 'SubAgent system prompt',
		source: 'manual',
		isFavorite: false,
		createdAt: now,
		updatedAt: now,
		...overrides
	};
}

export function createMockCommand(overrides: Partial<Command> = {}): Command {
	const id = overrides.id ?? nextId();
	return {
		id,
		name: `command-${id}`,
		description: `Command ${id} description`,
		content: 'Command content',
		source: 'manual',
		isFavorite: false,
		createdAt: now,
		updatedAt: now,
		...overrides
	};
}

export function createMockProfile(overrides: Partial<Profile> = {}): Profile {
	const id = overrides.id ?? nextId();
	return {
		id,
		name: `profile-${id}`,
		description: null,
		icon: null,
		isActive: false,
		createdAt: now,
		updatedAt: now,
		...overrides
	};
}

export function createMockStatusLine(overrides: Partial<StatusLine> = {}): StatusLine {
	const id = overrides.id ?? nextId();
	return {
		id,
		name: `statusline-${id}`,
		description: null,
		statuslineType: 'custom',
		packageName: null,
		installCommand: null,
		runCommand: null,
		rawCommand: null,
		padding: 1,
		isActive: false,
		segmentsJson: null,
		generatedScript: null,
		icon: null,
		author: null,
		homepageUrl: null,
		tags: null,
		source: 'manual',
		createdAt: now,
		updatedAt: now,
		...overrides
	};
}

export function createMockRepo(overrides: Partial<Repo> = {}): Repo {
	const id = overrides.id ?? nextId();
	return {
		id,
		name: `repo-${id}`,
		owner: 'test-owner',
		repo: `test-repo-${id}`,
		repoType: 'file_based',
		contentType: 'mcp',
		githubUrl: `https://github.com/test-owner/test-repo-${id}`,
		description: `Repo ${id} description`,
		isDefault: false,
		isEnabled: true,
		createdAt: now,
		updatedAt: now,
		...overrides
	};
}

export function createMockRepoItem(overrides: Partial<RepoItem> = {}): RepoItem {
	const id = overrides.id ?? nextId();
	return {
		id,
		repoId: 1,
		itemType: 'mcp',
		name: `item-${id}`,
		description: `Item ${id} description`,
		isImported: false,
		createdAt: now,
		updatedAt: now,
		...overrides
	};
}

export function createMockGalleryEntry(overrides: Partial<StatusLineGalleryEntry> = {}): StatusLineGalleryEntry {
	return {
		name: 'gallery-entry',
		description: 'A gallery entry',
		author: 'test-author',
		homepageUrl: null,
		installCommand: null,
		runCommand: null,
		packageName: null,
		icon: null,
		tags: null,
		previewText: null,
		...overrides
	};
}

export function createMockRegistryMcp(overrides: Partial<RegistryMcpEntry> = {}): RegistryMcpEntry {
	const name = overrides.name ?? `registry-mcp-${nextId()}`;
	return {
		registryId: overrides.registryId ?? name,
		name,
		description: 'A registry MCP',
		mcpType: 'stdio',
		command: 'npx',
		args: ['-y', `@example/${name}`],
		...overrides
	};
}

export function createMockSyncResult(overrides: Partial<SyncResult> = {}): SyncResult {
	return {
		added: 0,
		updated: 0,
		removed: 0,
		errors: [],
		...overrides
	};
}

export function createMockImportResult(overrides: Partial<ImportResult> = {}): ImportResult {
	return {
		success: true,
		itemType: 'mcp',
		itemId: 1,
		...overrides
	};
}
