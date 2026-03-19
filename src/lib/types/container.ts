export interface Container {
	id: number;
	name: string;
	description?: string;
	containerType: string;
	dockerHostId: number;
	dockerContainerId?: string;
	image?: string;
	dockerfile?: string;
	devcontainerJson?: string;
	env?: Record<string, string>;
	ports?: PortMapping[];
	volumes?: VolumeMapping[];
	mounts?: string[];
	features?: string[];
	postCreateCommand?: string;
	postStartCommand?: string;
	workingDir?: string;
	templateId?: string;
	icon?: string;
	tags?: string[];
	isFavorite: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface PortMapping {
	hostPort: number;
	containerPort: number;
	protocol?: string;
}

export interface VolumeMapping {
	hostPath: string;
	containerPath: string;
	readOnly?: boolean;
}

export interface ContainerStatus {
	containerId: number;
	dockerStatus: string;
	dockerContainerId?: string;
	startedAt?: string;
	finishedAt?: string;
	exitCode?: number;
	health?: string;
	cpuPercent?: number;
	memoryUsage?: number;
	memoryLimit?: number;
}

export interface DockerHost {
	id: number;
	name: string;
	hostType: string;
	connectionUri?: string;
	sshKeyPath?: string;
	tlsCaCert?: string;
	tlsCert?: string;
	tlsKey?: string;
	isDefault: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface CreateContainerRequest {
	name: string;
	description?: string;
	containerType: string;
	dockerHostId?: number;
	image?: string;
	dockerfile?: string;
	devcontainerJson?: string;
	env?: Record<string, string>;
	ports?: PortMapping[];
	volumes?: VolumeMapping[];
	mounts?: string[];
	features?: string[];
	postCreateCommand?: string;
	postStartCommand?: string;
	workingDir?: string;
	templateId?: string;
	icon?: string;
	tags?: string[];
}

export interface CreateDockerHostRequest {
	name: string;
	hostType: string;
	connectionUri?: string;
	sshKeyPath?: string;
	tlsCaCert?: string;
	tlsCert?: string;
	tlsKey?: string;
	isDefault?: boolean;
}

export interface ContainerTemplate {
	id: string;
	name: string;
	description?: string;
	containerType?: string;
	image?: string;
	dockerfile?: string;
	devcontainerJson?: string;
	env?: Record<string, string>;
	ports?: PortMapping[];
	volumes?: VolumeMapping[];
	features?: string[];
	postCreateCommand?: string;
	postStartCommand?: string;
	icon?: string;
	tags?: string[];
}

export interface ContainerLog {
	timestamp: string;
	message: string;
	stream: string;
}

export interface ContainerStats {
	cpuPercent: number;
	memoryUsageMb: number;
	memoryLimitMb?: number;
	networkRxBytes?: number;
	networkTxBytes?: number;
	blockReadBytes?: number;
	blockWriteBytes?: number;
}

export interface ExecResult {
	exitCode: number;
	stdout: string;
	stderr: string;
}

export interface ProjectContainer {
	containerId: number;
	isDefault: boolean;
}
