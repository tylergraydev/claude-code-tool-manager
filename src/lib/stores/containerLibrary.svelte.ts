class ContainerLibraryState {
	containers = $state<unknown[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
}

export const containerLibrary = new ContainerLibraryState();
