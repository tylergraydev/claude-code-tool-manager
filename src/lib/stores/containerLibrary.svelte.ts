class ContainerLibraryState {
	containers = $state<any[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');
	selectedType = $state('all');
	dockerAvailable = $state<boolean | null>(null);
	templates = $state<any[]>([]);
	dockerHosts = $state<any[]>([]);

	filteredContainers = $derived.by(() => {
		let items = this.containers;
		if (this.searchQuery) {
			const q = this.searchQuery.toLowerCase();
			items = items.filter((c: any) => c.name?.toLowerCase().includes(q));
		}
		if (this.selectedType !== 'all') {
			items = items.filter((c: any) => c.containerType === this.selectedType);
		}
		return items;
	});

	containerCount = $derived.by(() => ({
		total: this.containers.length,
		docker: this.containers.filter((c: any) => c.containerType === 'docker').length,
		devcontainer: this.containers.filter((c: any) => c.containerType === 'devcontainer').length,
		custom: this.containers.filter((c: any) => c.containerType === 'custom').length
	}));

	// Stub methods — container feature not yet fully implemented
	async load() { /* stub */ }
	async checkDocker() { this.dockerAvailable = null; }
	async refreshAllStatuses() { /* stub */ }
	startStatusPolling() { /* stub */ }
	stopStatusPolling() { /* stub */ }
	getStatus(_id: number): any { return null; }
	async create(_request: unknown) { throw new Error('Container feature not yet implemented'); }
	async update(_id: number, _request: unknown) { throw new Error('Container feature not yet implemented'); }
	async delete(_id: number) { throw new Error('Container feature not yet implemented'); }
	async startContainer(_id: number) { /* stub */ }
	async stopContainer(_id: number) { /* stub */ }
	async restartContainer(_id: number) { /* stub */ }
	async removeContainer(_id: number) { /* stub */ }
	async buildImage(_id: number) { /* stub */ }
	async toggleFavorite(_id: number) { /* stub */ }
	async exec(_id: number, _cmd: string[]) { return ''; }
	async fetchLogs(_id: number, _tail?: number) { return []; }
	async fetchStats(_id: number) { return null; }
	async loadTemplates() { /* stub */ }
	async createFromTemplate(_templateId: number, _name: string) { /* stub */ }
	async loadDockerHosts() { /* stub */ }
	async createDockerHost(_values: unknown) { /* stub */ }
	async deleteDockerHost(_id: number) { /* stub */ }
	async testDockerHost(_type: string, _uri?: string, _key?: string) { return null; }
	async getProjectContainers(_projectId: number) { return []; }
	async assignToProject(_projectId: number, _containerId: number) { /* stub */ }
	async removeFromProject(_projectId: number, _containerId: number) { /* stub */ }
	async setDefaultProjectContainer(_projectId: number, _containerId: number) { /* stub */ }
}

export const containerLibrary = new ContainerLibraryState();
