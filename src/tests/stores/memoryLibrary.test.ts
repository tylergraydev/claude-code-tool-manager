import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Memory Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('load', () => {
		it('should load memory files successfully', async () => {
			const mock = { user: { path: '/home/.claude/CLAUDE.md', content: '# Hello', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			expect(memoryLibrary.memoryFiles).toEqual(mock);
			expect(memoryLibrary.isLoading).toBe(false);
			expect(memoryLibrary.editedContent).toBeNull();
		});

		it('should handle load error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			expect(memoryLibrary.error).toBe('Error: fail');
		});
	});

	describe('currentFile', () => {
		it('should return user file by default', async () => {
			const mock = { user: { content: 'user content', exists: true }, project: { content: 'proj', exists: true }, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			expect(memoryLibrary.currentFile?.content).toBe('user content');
		});

		it('should return project file when scope is project', async () => {
			const mock = { user: { content: 'user', exists: true }, project: { content: 'proj', exists: true }, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			memoryLibrary.setScope('project');
			expect(memoryLibrary.currentFile?.content).toBe('proj');
		});

		it('should return local file when scope is local', async () => {
			const mock = { user: { content: 'user', exists: true }, project: null, local: { content: 'local', exists: true } };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			memoryLibrary.setScope('local');
			expect(memoryLibrary.currentFile?.content).toBe('local');
		});

		it('should return null when project scope has no file', async () => {
			const mock = { user: { content: 'user', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			memoryLibrary.setScope('project');
			expect(memoryLibrary.currentFile).toBeNull();
		});
	});

	describe('displayContent and hasUnsavedChanges', () => {
		it('should return current file content when no edits', async () => {
			const mock = { user: { content: 'original', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			expect(memoryLibrary.displayContent).toBe('original');
			expect(memoryLibrary.hasUnsavedChanges).toBe(false);
		});

		it('should return edited content when editing', async () => {
			const mock = { user: { content: 'original', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			memoryLibrary.setContent('modified');
			expect(memoryLibrary.displayContent).toBe('modified');
			expect(memoryLibrary.hasUnsavedChanges).toBe(true);
		});

		it('should not show unsaved when edit matches original', async () => {
			const mock = { user: { content: 'same', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			memoryLibrary.setContent('same');
			expect(memoryLibrary.hasUnsavedChanges).toBe(false);
		});

		it('should return empty when no file and no edits', async () => {
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			expect(memoryLibrary.displayContent).toBe('');
		});
	});

	describe('save', () => {
		it('should save and update state', async () => {
			const mock = { user: { content: 'original', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			const updated = { content: 'saved', exists: true };
			vi.mocked(invoke).mockResolvedValueOnce(updated);

			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			memoryLibrary.setContent('saved');
			await memoryLibrary.save();

			expect(memoryLibrary.editedContent).toBeNull();
			expect(memoryLibrary.memoryFiles?.user).toEqual(updated);
		});

		it('should do nothing when editedContent is null', async () => {
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.save();
			expect(invoke).not.toHaveBeenCalled();
		});

		it('should update project scope file', async () => {
			const mock = { user: { content: '', exists: true }, project: { content: '', exists: true }, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			vi.mocked(invoke).mockResolvedValueOnce({ content: 'new', exists: true });

			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			memoryLibrary.setScope('project');
			memoryLibrary.setContent('new');
			await memoryLibrary.save();

			expect(memoryLibrary.memoryFiles?.project?.content).toBe('new');
		});

		it('should update local scope file', async () => {
			const mock = { user: { content: '', exists: true }, project: null, local: { content: '', exists: true } };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			vi.mocked(invoke).mockResolvedValueOnce({ content: 'local', exists: true });

			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			memoryLibrary.setScope('local');
			memoryLibrary.setContent('local');
			await memoryLibrary.save();

			expect(memoryLibrary.memoryFiles?.local?.content).toBe('local');
		});

		it('should throw on save error', async () => {
			const mock = { user: { content: '', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			vi.mocked(invoke).mockRejectedValueOnce(new Error('save fail'));

			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			memoryLibrary.setContent('new');
			await expect(memoryLibrary.save()).rejects.toThrow('save fail');
		});
	});

	describe('createFile', () => {
		it('should create file for user scope', async () => {
			const mock = { user: { content: '', exists: false }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			vi.mocked(invoke).mockResolvedValueOnce({ content: '# New', exists: true });

			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			await memoryLibrary.createFile('# New');

			expect(memoryLibrary.memoryFiles?.user?.exists).toBe(true);
		});

		it('should create file for project scope', async () => {
			const mock = { user: { content: '', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			vi.mocked(invoke).mockResolvedValueOnce({ content: '', exists: true });

			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			memoryLibrary.setScope('project');
			await memoryLibrary.createFile();

			expect(memoryLibrary.memoryFiles?.project?.exists).toBe(true);
		});

		it('should create file for local scope', async () => {
			const mock = { user: { content: '', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			vi.mocked(invoke).mockResolvedValueOnce({ content: '', exists: true });

			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			memoryLibrary.setScope('local');
			await memoryLibrary.createFile();

			expect(memoryLibrary.memoryFiles?.local?.exists).toBe(true);
		});

		it('should throw on error', async () => {
			const mock = { user: { content: '', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			await expect(memoryLibrary.createFile()).rejects.toThrow('fail');
		});
	});

	describe('deleteFile', () => {
		it('should delete and reload', async () => {
			const mock = { user: { content: 'x', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			vi.mocked(invoke).mockResolvedValueOnce(undefined); // delete
			vi.mocked(invoke).mockResolvedValueOnce({ user: { content: '', exists: false }, project: null, local: null }); // reload

			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			await memoryLibrary.deleteFile();

			expect(invoke).toHaveBeenCalledWith('delete_memory_file', { scope: 'user', projectPath: null });
		});

		it('should throw on error', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: { content: '', exists: true }, project: null, local: null });
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			await expect(memoryLibrary.deleteFile()).rejects.toThrow('fail');
		});
	});

	describe('scope and project path', () => {
		it('should reset edited content on scope change', async () => {
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			memoryLibrary.setContent('test');
			memoryLibrary.setScope('project');
			expect(memoryLibrary.editedContent).toBeNull();
			expect(memoryLibrary.previewHtml).toBe('');
		});

		it('should reset to user scope when project path cleared from non-user scope', async () => {
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			memoryLibrary.setScope('project');
			memoryLibrary.setProjectPath(null);
			expect(memoryLibrary.selectedScope).toBe('user');
		});

		it('should not reset scope when clearing path from user scope', async () => {
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			memoryLibrary.setProjectPath(null);
			expect(memoryLibrary.selectedScope).toBe('user');
		});
	});

	describe('preview', () => {
		it('should toggle preview on', async () => {
			const mock = { user: { content: '# Title', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			vi.mocked(invoke).mockResolvedValueOnce('<h1>Title</h1>');

			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			memoryLibrary.togglePreview();

			expect(memoryLibrary.showPreview).toBe(true);
		});

		it('should toggle preview off', async () => {
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			memoryLibrary.togglePreview(); // on
			memoryLibrary.togglePreview(); // off
			expect(memoryLibrary.showPreview).toBe(false);
		});

		it('should set empty preview for empty content', async () => {
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.renderPreview();
			expect(memoryLibrary.previewHtml).toBe('');
		});

		it('should handle render error', async () => {
			const mock = { user: { content: '# Title', exists: true }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			vi.mocked(invoke).mockRejectedValueOnce(new Error('render fail'));

			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			await memoryLibrary.load();
			await memoryLibrary.renderPreview();

			expect(memoryLibrary.previewHtml).toContain('Failed to render preview');
		});
	});

	describe('discardChanges', () => {
		it('should reset edited content and preview', async () => {
			const { memoryLibrary } = await import('$lib/stores/memoryLibrary.svelte');
			memoryLibrary.setContent('test');
			memoryLibrary.discardChanges();
			expect(memoryLibrary.editedContent).toBeNull();
			expect(memoryLibrary.previewHtml).toBe('');
		});
	});
});
