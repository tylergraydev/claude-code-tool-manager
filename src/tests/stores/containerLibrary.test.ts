import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Container Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('load', () => {
		it('should load containers successfully', async () => {
			const mockContainers = [
				{ id: 1, name: 'test', containerType: 'docker', isFavorite: false }
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockContainers);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			expect(containerLibrary.containers).toEqual(mockContainers);
			expect(containerLibrary.isLoading).toBe(false);
			expect(containerLibrary.error).toBeNull();
		});

		it('should handle load error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Load failed'));

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			expect(containerLibrary.error).toBe('Error: Load failed');
			expect(containerLibrary.isLoading).toBe(false);
		});

		it('should set isLoading during load', async () => {
			let resolve: (v: unknown) => void;
			vi.mocked(invoke).mockReturnValueOnce(new Promise((r) => { resolve = r; }));

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			const p = containerLibrary.load();
			expect(containerLibrary.isLoading).toBe(true);

			resolve!([]);
			await p;
			expect(containerLibrary.isLoading).toBe(false);
		});
	});

	describe('create', () => {
		it('should create a container and add to list', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]); // load
			const newContainer = { id: 2, name: 'new', containerType: 'docker', isFavorite: false };
			vi.mocked(invoke).mockResolvedValueOnce(newContainer);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			const result = await containerLibrary.create({ name: 'new', containerType: 'docker' } as any);
			expect(result).toEqual(newContainer);
			expect(containerLibrary.containers).toHaveLength(1);
		});
	});

	describe('update', () => {
		it('should update a container in the list', async () => {
			const original = { id: 1, name: 'old', containerType: 'docker', isFavorite: false };
			vi.mocked(invoke).mockResolvedValueOnce([original]); // load
			const updated = { ...original, name: 'updated' };
			vi.mocked(invoke).mockResolvedValueOnce(updated);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			const result = await containerLibrary.update(1, { name: 'updated', containerType: 'docker' } as any);
			expect(result.name).toBe('updated');
			expect(containerLibrary.containers[0].name).toBe('updated');
		});
	});

	describe('delete', () => {
		it('should remove container from list', async () => {
			const containers = [
				{ id: 1, name: 'a', containerType: 'docker', isFavorite: false },
				{ id: 2, name: 'b', containerType: 'docker', isFavorite: false }
			];
			vi.mocked(invoke).mockResolvedValueOnce(containers);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			vi.mocked(invoke).mockResolvedValueOnce(undefined);
			await containerLibrary.delete(1);

			expect(containerLibrary.containers).toHaveLength(1);
			expect(containerLibrary.containers[0].id).toBe(2);
		});
	});

	describe('toggleFavorite', () => {
		it('should toggle favorite state', async () => {
			const containers = [{ id: 1, name: 'a', containerType: 'docker', isFavorite: false }];
			vi.mocked(invoke).mockResolvedValueOnce(containers);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			vi.mocked(invoke).mockResolvedValueOnce(undefined);
			await containerLibrary.toggleFavorite(1);

			expect(containerLibrary.containers[0].isFavorite).toBe(true);
		});
	});

	describe('checkDocker', () => {
		it('should return true when docker available', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(true);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			const result = await containerLibrary.checkDocker();

			expect(result).toBe(true);
			expect(containerLibrary.dockerAvailable).toBe(true);
		});

		it('should return false when docker unavailable', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('not found'));

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			const result = await containerLibrary.checkDocker();

			expect(result).toBe(false);
			expect(containerLibrary.dockerAvailable).toBe(false);
		});
	});

	describe('lifecycle operations', () => {
		it('should build image', async () => {
			vi.mocked(invoke).mockResolvedValueOnce('image-id');

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			const result = await containerLibrary.buildImage(1);

			expect(result).toBe('image-id');
			expect(invoke).toHaveBeenCalledWith('build_container_image', { id: 1 });
		});

		it('should start container and refresh status', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined); // start
			vi.mocked(invoke).mockResolvedValueOnce({ state: 'running' }); // refreshStatus

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.startContainer(1);

			expect(invoke).toHaveBeenCalledWith('start_container_cmd', { id: 1 });
		});

		it('should stop container and refresh status', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);
			vi.mocked(invoke).mockResolvedValueOnce({ state: 'stopped' });

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.stopContainer(1);

			expect(invoke).toHaveBeenCalledWith('stop_container_cmd', { id: 1 });
		});

		it('should restart container and refresh status', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);
			vi.mocked(invoke).mockResolvedValueOnce({ state: 'running' });

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.restartContainer(1);

			expect(invoke).toHaveBeenCalledWith('restart_container_cmd', { id: 1 });
		});

		it('should remove container, refresh status, and reload', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined); // remove
			vi.mocked(invoke).mockResolvedValueOnce({ state: 'removed' }); // refreshStatus
			vi.mocked(invoke).mockResolvedValueOnce([]); // reload

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.removeContainer(1);

			expect(invoke).toHaveBeenCalledWith('remove_container_cmd', { id: 1 });
		});
	});

	describe('status operations', () => {
		it('should refresh status for a container', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ state: 'running' });

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			const status = await containerLibrary.refreshStatus(1);

			expect(status).toEqual({ state: 'running' });
			expect(containerLibrary.getStatus(1)).toEqual({ state: 'running' });
		});

		it('should refresh all statuses', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([
				{ id: 1, status: { state: 'running' } },
				{ id: 2, status: { state: 'stopped' } }
			]);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.refreshAllStatuses();

			expect(containerLibrary.getStatus(1)).toEqual({ state: 'running' });
			expect(containerLibrary.getStatus(2)).toEqual({ state: 'stopped' });
		});

		it('should handle refreshAllStatuses error gracefully', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.refreshAllStatuses();
			// Should not throw
		});

		it('should return undefined for unknown container status', async () => {
			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			expect(containerLibrary.getStatus(999)).toBeUndefined();
		});
	});

	describe('status polling', () => {
		it('should start and stop polling', async () => {
			vi.useFakeTimers();
			vi.mocked(invoke).mockResolvedValue([]);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');

			// Mock document.visibilityState
			Object.defineProperty(document, 'visibilityState', { value: 'visible', writable: true });

			containerLibrary.startStatusPolling();
			// Starting again should be a no-op
			containerLibrary.startStatusPolling();

			vi.advanceTimersByTime(5000);

			containerLibrary.stopStatusPolling();
			// Stopping again should be safe
			containerLibrary.stopStatusPolling();

			vi.useRealTimers();
		});

		it('should not refresh when document is hidden', async () => {
			vi.useFakeTimers();

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');

			Object.defineProperty(document, 'visibilityState', { value: 'hidden', writable: true });

			containerLibrary.startStatusPolling();
			vi.advanceTimersByTime(5000);

			// invoke should not have been called for refreshAllStatuses
			expect(invoke).not.toHaveBeenCalledWith('get_all_container_statuses');

			containerLibrary.stopStatusPolling();
			vi.useRealTimers();
		});
	});

	describe('logs and stats', () => {
		it('should fetch logs', async () => {
			const mockLogs = [{ timestamp: '2024-01-01', message: 'hello', stream: 'stdout' }];
			vi.mocked(invoke).mockResolvedValueOnce(mockLogs);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			const logs = await containerLibrary.fetchLogs(1, 100, 0);

			expect(logs).toEqual(mockLogs);
			expect(invoke).toHaveBeenCalledWith('get_container_logs_cmd', { id: 1, tail: 100, since: 0 });
		});

		it('should fetch stats', async () => {
			const mockStats = { cpuPercent: 50, memoryUsageMb: 100 };
			vi.mocked(invoke).mockResolvedValueOnce(mockStats);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			const stats = await containerLibrary.fetchStats(1);

			expect(stats).toEqual(mockStats);
		});

		it('should exec command', async () => {
			const mockResult = { exitCode: 0, stdout: 'hello', stderr: '' };
			vi.mocked(invoke).mockResolvedValueOnce(mockResult);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			const result = await containerLibrary.exec(1, ['echo', 'hello']);

			expect(result).toEqual(mockResult);
			expect(invoke).toHaveBeenCalledWith('exec_in_container_cmd', { id: 1, command: ['echo', 'hello'] });
		});
	});

	describe('templates', () => {
		it('should load templates', async () => {
			const mockTemplates = [{ id: 'node', name: 'Node.js' }];
			vi.mocked(invoke).mockResolvedValueOnce(mockTemplates);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.loadTemplates();

			expect(containerLibrary.templates).toEqual(mockTemplates);
		});

		it('should handle load templates error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.loadTemplates();
			// Should not throw
		});

		it('should create from template', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]); // load
			const newContainer = { id: 1, name: 'from-template' };
			vi.mocked(invoke).mockResolvedValueOnce(newContainer);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			const result = await containerLibrary.createFromTemplate('node', 'my-node');
			expect(result).toEqual(newContainer);
			expect(containerLibrary.containers).toHaveLength(1);
		});
	});

	describe('docker hosts', () => {
		it('should load docker hosts', async () => {
			const hosts = [{ id: 1, name: 'local', hostType: 'local' }];
			vi.mocked(invoke).mockResolvedValueOnce(hosts);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.loadDockerHosts();

			expect(containerLibrary.dockerHosts).toEqual(hosts);
		});

		it('should handle load docker hosts error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.loadDockerHosts();
			// Should not throw
		});

		it('should create docker host', async () => {
			const newHost = { id: 1, name: 'remote', hostType: 'ssh' };
			vi.mocked(invoke).mockResolvedValueOnce(newHost);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			const result = await containerLibrary.createDockerHost({ name: 'remote', hostType: 'ssh' } as any);
			expect(result).toEqual(newHost);
			expect(containerLibrary.dockerHosts).toHaveLength(1);
		});

		it('should update docker host', async () => {
			const newHost = { id: 1, name: 'old', hostType: 'local' };
			vi.mocked(invoke).mockResolvedValueOnce(newHost); // createDockerHost

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.createDockerHost({ name: 'old', hostType: 'local' } as any);

			const updated = { id: 1, name: 'updated', hostType: 'ssh' };
			vi.mocked(invoke).mockResolvedValueOnce(updated);

			const result = await containerLibrary.updateDockerHost(1, { name: 'updated', hostType: 'ssh' } as any);
			expect(result.name).toBe('updated');
			expect(containerLibrary.dockerHosts[0].name).toBe('updated');
		});

		it('should delete docker host', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ id: 1, name: 'a' });
			vi.mocked(invoke).mockResolvedValueOnce({ id: 2, name: 'b' });

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.createDockerHost({ name: 'a' } as any);
			await containerLibrary.createDockerHost({ name: 'b' } as any);

			vi.mocked(invoke).mockResolvedValueOnce(undefined);
			await containerLibrary.deleteDockerHost(1);

			expect(containerLibrary.dockerHosts).toHaveLength(1);
			expect(containerLibrary.dockerHosts[0].id).toBe(2);
		});

		it('should test docker host', async () => {
			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			vi.mocked(invoke).mockResolvedValueOnce('Connected successfully');

			const result = await containerLibrary.testDockerHost('ssh', 'ssh://host', '/path/key');

			expect(result).toBe('Connected successfully');
			expect(invoke).toHaveBeenCalledWith('test_docker_host', {
				hostType: 'ssh',
				connectionUri: 'ssh://host',
				sshKeyPath: '/path/key'
			});
		});
	});

	describe('project containers', () => {
		it('should assign container to project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.assignToProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('assign_container_to_project', { projectId: 1, containerId: 2 });
		});

		it('should remove container from project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.removeFromProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('remove_container_from_project', { projectId: 1, containerId: 2 });
		});

		it('should get project containers', async () => {
			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			const mockPc = [{ containerId: 1, isDefault: true }];
			vi.mocked(invoke).mockResolvedValueOnce(mockPc);

			const result = await containerLibrary.getProjectContainers(1);

			expect(result).toEqual(mockPc);
		});

		it('should set default project container', async () => {
			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			await containerLibrary.setDefaultProjectContainer(1, 2);

			expect(invoke).toHaveBeenCalledWith('set_default_project_container', { projectId: 1, containerId: 2 });
		});
	});

	describe('derived state', () => {
		it('should filter containers by search query', async () => {
			const containers = [
				{ id: 1, name: 'node-app', description: 'Node app', containerType: 'docker', isFavorite: false, tags: ['node'] },
				{ id: 2, name: 'python-api', description: 'Python API', containerType: 'docker', isFavorite: false, tags: ['python'] }
			];
			vi.mocked(invoke).mockResolvedValueOnce(containers);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			containerLibrary.searchQuery = 'node';
			expect(containerLibrary.filteredContainers).toHaveLength(1);
			expect(containerLibrary.filteredContainers[0].name).toBe('node-app');
		});

		it('should filter containers by description', async () => {
			const containers = [
				{ id: 1, name: 'app', description: 'Special tool', containerType: 'docker', isFavorite: false, tags: [] },
				{ id: 2, name: 'other', description: null, containerType: 'docker', isFavorite: false, tags: null }
			];
			vi.mocked(invoke).mockResolvedValueOnce(containers);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			containerLibrary.searchQuery = 'special';
			expect(containerLibrary.filteredContainers).toHaveLength(1);
		});

		it('should filter containers by tag', async () => {
			const containers = [
				{ id: 1, name: 'app', description: null, containerType: 'docker', isFavorite: false, tags: ['web', 'frontend'] },
				{ id: 2, name: 'other', description: null, containerType: 'docker', isFavorite: false, tags: [] }
			];
			vi.mocked(invoke).mockResolvedValueOnce(containers);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			containerLibrary.searchQuery = 'frontend';
			expect(containerLibrary.filteredContainers).toHaveLength(1);
		});

		it('should filter by container type', async () => {
			const containers = [
				{ id: 1, name: 'a', containerType: 'docker', isFavorite: false },
				{ id: 2, name: 'b', containerType: 'devcontainer', isFavorite: false },
				{ id: 3, name: 'c', containerType: 'custom', isFavorite: false }
			];
			vi.mocked(invoke).mockResolvedValueOnce(containers);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			containerLibrary.selectedType = 'docker';
			expect(containerLibrary.filteredContainers).toHaveLength(1);
		});

		it('should sort favorites first', async () => {
			const containers = [
				{ id: 1, name: 'b', containerType: 'docker', isFavorite: false },
				{ id: 2, name: 'a', containerType: 'docker', isFavorite: true }
			];
			vi.mocked(invoke).mockResolvedValueOnce(containers);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			expect(containerLibrary.filteredContainers[0].id).toBe(2);
		});

		it('should compute container counts', async () => {
			const containers = [
				{ id: 1, name: 'a', containerType: 'docker', isFavorite: false },
				{ id: 2, name: 'b', containerType: 'docker', isFavorite: false },
				{ id: 3, name: 'c', containerType: 'devcontainer', isFavorite: false },
				{ id: 4, name: 'd', containerType: 'custom', isFavorite: false }
			];
			vi.mocked(invoke).mockResolvedValueOnce(containers);

			const { containerLibrary } = await import('$lib/stores/containerLibrary.svelte');
			await containerLibrary.load();

			expect(containerLibrary.containerCount).toEqual({
				total: 4,
				docker: 2,
				devcontainer: 1,
				custom: 1
			});
		});
	});
});
