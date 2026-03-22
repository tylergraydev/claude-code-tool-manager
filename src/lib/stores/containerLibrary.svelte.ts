import { invoke } from '@tauri-apps/api/core';
import { notifications } from '$lib/stores/notifications.svelte';
import type {
	Container,
	CreateContainerRequest,
	CreateDockerHostRequest,
	ContainerTemplate,
	ContainerLog,
	ContainerStats,
	ExecResult,
	DockerHost,
	ProjectContainer
} from '$lib/types';

class ContainerLibraryState {
	containers = $state<Container[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');
	selectedType = $state<string>('all');
	dockerAvailable = $state(false);
	templates = $state<ContainerTemplate[]>([]);
	dockerHosts = $state<DockerHost[]>([]);

	private statuses = $state<Map<number, unknown>>(new Map());
	private pollingInterval: ReturnType<typeof setInterval> | null = null;

	// Callback for when a container stops unexpectedly — set by the page to open detail view
	onContainerStopped: ((containerId: number) => void) | null = null;

	filteredContainers = $derived.by(() => {
		let result = this.containers;

		if (this.searchQuery) {
			const query = this.searchQuery.toLowerCase();
			result = result.filter(
				(c) =>
					c.name.toLowerCase().includes(query) ||
					c.description?.toLowerCase().includes(query) ||
					c.tags?.some((t) => t.toLowerCase().includes(query))
			);
		}

		if (this.selectedType !== 'all') {
			result = result.filter((c) => c.containerType === this.selectedType);
		}

		return [...result].sort((a, b) => {
			if (a.isFavorite !== b.isFavorite) {
				return a.isFavorite ? -1 : 1;
			}
			return a.name.localeCompare(b.name);
		});
	});

	containerCount = $derived.by(() => {
		let docker = 0,
			devcontainer = 0,
			custom = 0;
		for (const c of this.containers) {
			if (c.containerType === 'docker') docker++;
			else if (c.containerType === 'devcontainer') devcontainer++;
			else if (c.containerType === 'custom') custom++;
		}
		return { total: this.containers.length, docker, devcontainer, custom };
	});

	async load() {
		this.isLoading = true;
		this.error = null;
		try {
			this.containers = await invoke<Container[]>('get_all_containers');
		} catch (e) {
			this.error = String(e);
		} finally {
			this.isLoading = false;
		}
	}

	async create(request: CreateContainerRequest): Promise<Container> {
		const container = await invoke<Container>('create_container', { container: request });
		this.containers = [...this.containers, container];
		return container;
	}

	async update(id: number, request: CreateContainerRequest): Promise<Container> {
		const container = await invoke<Container>('update_container', { id, container: request });
		this.containers = this.containers.map((c) => (c.id === id ? container : c));
		return container;
	}

	async delete(id: number): Promise<void> {
		await invoke('delete_container', { id });
		this.containers = this.containers.filter((c) => c.id !== id);
	}

	async toggleFavorite(id: number): Promise<void> {
		await invoke('toggle_container_favorite', { id });
		this.containers = this.containers.map((c) =>
			c.id === id ? { ...c, isFavorite: !c.isFavorite } : c
		);
	}

	async checkDocker(): Promise<boolean> {
		try {
			const available = await invoke<boolean>('check_docker_available');
			this.dockerAvailable = available;
			return available;
		} catch {
			this.dockerAvailable = false;
			return false;
		}
	}

	// Lifecycle operations
	async buildImage(id: number): Promise<string> {
		return await invoke<string>('build_container_image', { id });
	}

	async startContainer(id: number): Promise<void> {
		const container = this.containers.find(c => c.id === id);
		await invoke('start_container_cmd', { id });
		await this.refreshStatus(id);
		notifications.success(`Container "${container?.name || id}" started`);
	}

	async stopContainer(id: number): Promise<void> {
		const container = this.containers.find(c => c.id === id);
		await invoke('stop_container_cmd', { id });
		await this.refreshStatus(id);
		notifications.info(`Container "${container?.name || id}" stopped`);
	}

	async restartContainer(id: number): Promise<void> {
		const container = this.containers.find(c => c.id === id);
		await invoke('restart_container_cmd', { id });
		await this.refreshStatus(id);
		notifications.success(`Container "${container?.name || id}" restarted`);
	}

	async removeContainer(id: number): Promise<void> {
		await invoke('remove_container_cmd', { id });
		await this.refreshStatus(id);
		await this.load();
	}

	// Status operations
	async refreshStatus(id: number): Promise<unknown> {
		const status = await invoke('get_container_status', { id });
		this.statuses = new Map(this.statuses).set(id, status);
		return status;
	}

	async refreshAllStatuses(): Promise<void> {
		try {
			const results = await invoke<Array<{ id: number; name: string; status: { dockerStatus: string; exitCode?: number } }>>(
				'get_all_container_statuses'
			);
			const newStatuses = new Map(this.statuses);
			for (const r of results) {
				const oldStatus = this.statuses.get(r.id) as { dockerStatus?: string } | undefined;
				const oldDockerStatus = oldStatus?.dockerStatus;
				const newDockerStatus = r.status.dockerStatus;

				// Detect unexpected stop: was running, now exited/stopped
				if (oldDockerStatus === 'running' && (newDockerStatus === 'exited' || newDockerStatus === 'stopped')) {
					const exitCode = r.status.exitCode;
					const exitReason = exitCode === 0 ? 'exited cleanly' :
						exitCode === 137 ? 'killed (OOM or SIGKILL)' :
						exitCode === 143 ? 'terminated (SIGTERM)' :
						exitCode === 1 ? 'exited with error' :
						exitCode !== undefined && exitCode !== null ? `exit code ${exitCode}` : 'unknown reason';
					const containerId = r.id;
					const onViewLogs = this.onContainerStopped;

					notifications.add('warning', `Container "${r.name}" stopped`, {
						detail: `Reason: ${exitReason}. Check logs for more details.`,
						duration: 10000,
						action: onViewLogs ? {
							label: 'View Logs',
							onclick: () => onViewLogs(containerId),
						} : undefined,
					});
				}

				newStatuses.set(r.id, r.status);
			}
			this.statuses = newStatuses;
		} catch {
			// Silently handle errors
		}
	}

	getStatus(id: number): unknown {
		return this.statuses.get(id);
	}

	// Status polling
	startStatusPolling(): void {
		if (this.pollingInterval !== null) return;
		this.pollingInterval = setInterval(() => {
			if (document.visibilityState === 'visible') {
				this.refreshAllStatuses();
			}
		}, 5000);
	}

	stopStatusPolling(): void {
		if (this.pollingInterval !== null) {
			clearInterval(this.pollingInterval);
			this.pollingInterval = null;
		}
	}

	// Logs and stats
	async fetchLogs(id: number, tail: number, since: number): Promise<ContainerLog[]> {
		return await invoke<ContainerLog[]>('get_container_logs_cmd', { id, tail, since });
	}

	async fetchStats(id: number): Promise<ContainerStats> {
		return await invoke<ContainerStats>('get_container_stats_cmd', { id });
	}

	async exec(id: number, command: string[]): Promise<ExecResult> {
		return await invoke<ExecResult>('exec_in_container_cmd', { id, command });
	}

	// Templates
	async loadTemplates(): Promise<void> {
		try {
			this.templates = await invoke<ContainerTemplate[]>('get_container_templates');
		} catch {
			// Silently handle errors
		}
	}

	async createFromTemplate(templateId: string, name: string): Promise<Container> {
		const container = await invoke<Container>('create_container_from_template', {
			templateId,
			name
		});
		this.containers = [...this.containers, container];
		return container;
	}

	// Docker hosts
	async loadDockerHosts(): Promise<void> {
		try {
			this.dockerHosts = await invoke<DockerHost[]>('get_all_docker_hosts');
		} catch {
			// Silently handle errors
		}
	}

	async createDockerHost(request: CreateDockerHostRequest): Promise<DockerHost> {
		const host = await invoke<DockerHost>('create_docker_host', { host: request });
		this.dockerHosts = [...this.dockerHosts, host];
		return host;
	}

	async updateDockerHost(id: number, request: CreateDockerHostRequest): Promise<DockerHost> {
		const host = await invoke<DockerHost>('update_docker_host', { id, host: request });
		this.dockerHosts = this.dockerHosts.map((h) => (h.id === id ? host : h));
		return host;
	}

	async deleteDockerHost(id: number): Promise<void> {
		await invoke('delete_docker_host', { id });
		this.dockerHosts = this.dockerHosts.filter((h) => h.id !== id);
	}

	async testDockerHost(
		hostType: string,
		connectionUri: string,
		sshKeyPath: string
	): Promise<string> {
		return await invoke<string>('test_docker_host', { hostType, connectionUri, sshKeyPath });
	}

	// Project containers
	async assignToProject(projectId: number, containerId: number): Promise<void> {
		await invoke('assign_container_to_project', { projectId, containerId });
	}

	async removeFromProject(projectId: number, containerId: number): Promise<void> {
		await invoke('remove_container_from_project', { projectId, containerId });
	}

	async getProjectContainers(projectId: number): Promise<ProjectContainer[]> {
		return await invoke<ProjectContainer[]>('get_project_containers', { projectId });
	}

	async setDefaultProjectContainer(projectId: number, containerId: number): Promise<void> {
		await invoke('set_default_project_container', { projectId, containerId });
	}
}

export const containerLibrary = new ContainerLibraryState();
