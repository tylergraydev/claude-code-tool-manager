import { describe, it, expect, vi, beforeAll, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	containerLibrary: {
		containers: [],
		dockerHosts: [],
		templates: [],
		filteredContainers: [],
		isLoading: false,
		searchQuery: '',
		load: vi.fn(),
		loadHosts: vi.fn(),
		loadTemplates: vi.fn(),
		getStatus: vi.fn().mockReturnValue({ dockerStatus: 'stopped' }),
		buildImage: vi.fn(),
		startContainer: vi.fn(),
		stopContainer: vi.fn(),
		restartContainer: vi.fn(),
		removeContainer: vi.fn(),
		toggleFavorite: vi.fn(),
		fetchLogs: vi.fn().mockResolvedValue([]),
		fetchStats: vi.fn().mockResolvedValue(null),
		exec: vi.fn().mockResolvedValue({ stdout: '', stderr: '' }),
		testDockerHost: vi.fn(),
		deleteDockerHost: vi.fn(),
		getProjectContainers: vi.fn().mockResolvedValue([]),
		assignToProject: vi.fn(),
		removeFromProject: vi.fn(),
		setDefaultProjectContainer: vi.fn()
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

// ──────────────────────────────────────────────────────────
// ContainerStatus
// ──────────────────────────────────────────────────────────
describe('ContainerStatus Component', () => {
	let ContainerStatus: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/containers/ContainerStatus.svelte');
		ContainerStatus = mod.default;
	});

	it('should render Running status', () => {
		render(ContainerStatus, { props: { status: 'running' } });
		expect(screen.getByText('Running')).toBeInTheDocument();
	});

	it('should render Stopped status', () => {
		render(ContainerStatus, { props: { status: 'stopped' } });
		expect(screen.getByText('Stopped')).toBeInTheDocument();
	});

	it('should render Exited status', () => {
		render(ContainerStatus, { props: { status: 'exited' } });
		expect(screen.getByText('Exited')).toBeInTheDocument();
	});

	it('should render Created status', () => {
		render(ContainerStatus, { props: { status: 'created' } });
		expect(screen.getByText('Created')).toBeInTheDocument();
	});

	it('should render Not Created status', () => {
		render(ContainerStatus, { props: { status: 'not_created' } });
		expect(screen.getByText('Not Created')).toBeInTheDocument();
	});

	it('should render Unknown status', () => {
		render(ContainerStatus, { props: { status: 'unknown' } });
		expect(screen.getByText('Unknown')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// ContainerActions
// ──────────────────────────────────────────────────────────
describe('ContainerActions Component', () => {
	let ContainerActions: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/containers/ContainerActions.svelte');
		ContainerActions = mod.default;
	});

	it('should show Build and Start buttons when status is not_created', () => {
		const onBuild = vi.fn();
		const onStart = vi.fn();
		render(ContainerActions, {
			props: { status: 'not_created', onBuild, onStart }
		});
		expect(screen.getByTitle('Build')).toBeInTheDocument();
		expect(screen.getByTitle('Start')).toBeInTheDocument();
	});

	it('should not show Build when onBuild is undefined and status is not_created', () => {
		render(ContainerActions, {
			props: { status: 'not_created', onStart: vi.fn() }
		});
		expect(screen.queryByTitle('Build')).not.toBeInTheDocument();
		expect(screen.getByTitle('Start')).toBeInTheDocument();
	});

	it('should show Stop and Restart buttons when status is running', () => {
		render(ContainerActions, {
			props: { status: 'running', onStop: vi.fn(), onRestart: vi.fn() }
		});
		expect(screen.getByTitle('Stop')).toBeInTheDocument();
		expect(screen.getByTitle('Restart')).toBeInTheDocument();
	});

	it('should show Start button for stopped status (else branch)', () => {
		render(ContainerActions, {
			props: { status: 'stopped', onStart: vi.fn() }
		});
		expect(screen.getByTitle('Start')).toBeInTheDocument();
	});

	it('should show Remove button when status is stopped and onRemove provided', () => {
		render(ContainerActions, {
			props: { status: 'stopped', onStart: vi.fn(), onRemove: vi.fn() }
		});
		expect(screen.getByTitle('Remove Docker container')).toBeInTheDocument();
	});

	it('should not show Remove button when status is running', () => {
		render(ContainerActions, {
			props: { status: 'running', onStop: vi.fn(), onRemove: vi.fn() }
		});
		expect(screen.queryByTitle('Remove Docker container')).not.toBeInTheDocument();
	});

	it('should not show Remove button when status is not_created', () => {
		render(ContainerActions, {
			props: { status: 'not_created', onStart: vi.fn(), onRemove: vi.fn() }
		});
		expect(screen.queryByTitle('Remove Docker container')).not.toBeInTheDocument();
	});

	it('should show Remove for exited status', () => {
		render(ContainerActions, {
			props: { status: 'exited', onStart: vi.fn(), onRemove: vi.fn() }
		});
		expect(screen.getByTitle('Remove Docker container')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// ContainerCard
// ──────────────────────────────────────────────────────────
describe('ContainerCard Component', () => {
	let ContainerCard: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/containers/ContainerCard.svelte');
		ContainerCard = mod.default;
	});

	const baseContainer = {
		id: 1,
		name: 'My Container',
		description: 'A test container',
		containerType: 'docker',
		image: 'node:20',
		icon: '',
		isFavorite: false,
		dockerfile: '',
		ports: [],
		volumes: [],
		env: {},
		tags: []
	};

	it('should render container name', () => {
		render(ContainerCard, {
			props: { container: baseContainer, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.getByText('My Container')).toBeInTheDocument();
	});

	it('should render description when present', () => {
		render(ContainerCard, {
			props: { container: baseContainer, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.getByText('A test container')).toBeInTheDocument();
	});

	it('should not render description when absent', () => {
		const noDesc = { ...baseContainer, description: '' };
		render(ContainerCard, {
			props: { container: noDesc, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.queryByText('A test container')).not.toBeInTheDocument();
	});

	it('should render type label Docker', () => {
		render(ContainerCard, {
			props: { container: baseContainer, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.getByText('Docker')).toBeInTheDocument();
	});

	it('should render type label Dev Container', () => {
		const devContainer = { ...baseContainer, containerType: 'devcontainer' };
		render(ContainerCard, {
			props: { container: devContainer, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.getByText('Dev Container')).toBeInTheDocument();
	});

	it('should render type label Custom', () => {
		const customContainer = { ...baseContainer, containerType: 'custom' };
		render(ContainerCard, {
			props: { container: customContainer, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.getByText('Custom')).toBeInTheDocument();
	});

	it('should render image name when present', () => {
		render(ContainerCard, {
			props: { container: baseContainer, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.getByText('node:20')).toBeInTheDocument();
	});

	it('should not render image when absent', () => {
		const noImage = { ...baseContainer, image: '' };
		render(ContainerCard, {
			props: { container: noImage, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.queryByText('node:20')).not.toBeInTheDocument();
	});

	it('should render favorite button with correct aria-label when not favorite', () => {
		render(ContainerCard, {
			props: { container: baseContainer, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.getByLabelText('Add to favorites')).toBeInTheDocument();
	});

	it('should render favorite button with correct aria-label when favorite', () => {
		const fav = { ...baseContainer, isFavorite: true };
		render(ContainerCard, {
			props: { container: fav, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.getByLabelText('Remove from favorites')).toBeInTheDocument();
	});

	it('should render edit and delete buttons', () => {
		render(ContainerCard, {
			props: { container: baseContainer, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.getByLabelText('Edit container')).toBeInTheDocument();
		expect(screen.getByLabelText('Delete container')).toBeInTheDocument();
	});

	it('should use default icon when container has no icon', () => {
		const { container: rendered } = render(ContainerCard, {
			props: { container: baseContainer, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		// Default icon is the box emoji
		expect(document.body.textContent).toContain('\u{1F4E6}');
	});

	it('should render custom icon when set', () => {
		const withIcon = { ...baseContainer, icon: '\u{1F680}' };
		render(ContainerCard, {
			props: { container: withIcon, onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(document.body.textContent).toContain('\u{1F680}');
	});
});

// ──────────────────────────────────────────────────────────
// ContainerForm
// ──────────────────────────────────────────────────────────
describe('ContainerForm Component', () => {
	let ContainerForm: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/containers/ContainerForm.svelte');
		ContainerForm = mod.default;
	});

	it('should render Create Container button for new form', () => {
		render(ContainerForm, {
			props: { onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByText('Create Container')).toBeInTheDocument();
	});

	it('should render Update Container button when editing existing', () => {
		const existing = {
			id: 1,
			name: 'Existing',
			containerType: 'docker',
			ports: [],
			volumes: [],
			env: {}
		};
		render(ContainerForm, {
			props: { container: existing, onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByText('Update Container')).toBeInTheDocument();
	});

	it('should render Cancel button', () => {
		render(ContainerForm, {
			props: { onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByText('Cancel')).toBeInTheDocument();
	});

	it('should render Name label', () => {
		render(ContainerForm, {
			props: { onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByText('Name *')).toBeInTheDocument();
	});

	it('should render all form sections', () => {
		render(ContainerForm, {
			props: { onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByText('Description')).toBeInTheDocument();
		expect(screen.getByText('Image')).toBeInTheDocument();
		expect(screen.getByText('Working Directory')).toBeInTheDocument();
		expect(screen.getByText('Dockerfile')).toBeInTheDocument();
		expect(screen.getByText('Environment Variables')).toBeInTheDocument();
		expect(screen.getByText('Post Create Command')).toBeInTheDocument();
		expect(screen.getByText('Post Start Command')).toBeInTheDocument();
		expect(screen.getByText('Icon (emoji)')).toBeInTheDocument();
		expect(screen.getByText('Tags')).toBeInTheDocument();
	});

	it('should render Port Mappings and Volume Mappings sub-editors', () => {
		render(ContainerForm, {
			props: { onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByText('Port Mappings')).toBeInTheDocument();
		expect(screen.getByText('Volume Mappings')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// ContainerList
// ──────────────────────────────────────────────────────────
describe('ContainerList Component', () => {
	let ContainerList: any;
	let containerLibrary: any;

	beforeAll(async () => {
		const stores = await import('$lib/stores');
		containerLibrary = (stores as any).containerLibrary;
		const mod = await import('$lib/components/containers/ContainerList.svelte');
		ContainerList = mod.default;
	});

	beforeEach(() => {
		containerLibrary.isLoading = false;
		containerLibrary.filteredContainers = [];
		containerLibrary.searchQuery = '';
	});

	it('should render empty state when no containers', () => {
		render(ContainerList, {
			props: { onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.getByText('No containers yet')).toBeInTheDocument();
		expect(screen.getByText('Create a container or use a template to get started')).toBeInTheDocument();
	});

	it('should render search empty state when search has no results', () => {
		containerLibrary.searchQuery = 'xyz';
		containerLibrary.filteredContainers = [];
		render(ContainerList, {
			props: { onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.getByText('No matching containers')).toBeInTheDocument();
		expect(screen.getByText('Try a different search term')).toBeInTheDocument();
	});

	it('should render loading spinner when isLoading', () => {
		containerLibrary.isLoading = true;
		const { container } = render(ContainerList, {
			props: { onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(container.querySelector('.animate-spin')).toBeTruthy();
	});

	it('should render container cards when containers exist', () => {
		containerLibrary.filteredContainers = [
			{ id: 1, name: 'Test Container', containerType: 'docker', ports: [], volumes: [], env: {} }
		];
		render(ContainerList, {
			props: { onEdit: vi.fn(), onDelete: vi.fn() }
		});
		expect(screen.getByText('Test Container')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// ContainerLogs
// ──────────────────────────────────────────────────────────
describe('ContainerLogs Component', () => {
	let ContainerLogs: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/containers/ContainerLogs.svelte');
		ContainerLogs = mod.default;
	});

	it('should render no logs message initially', () => {
		render(ContainerLogs, { props: { containerId: 1 } });
		expect(screen.getByText('No logs available')).toBeInTheDocument();
	});

	it('should render auto-scroll checkbox', () => {
		render(ContainerLogs, { props: { containerId: 1 } });
		expect(screen.getByText('Auto-scroll')).toBeInTheDocument();
	});

	it('should render tail lines select options', () => {
		render(ContainerLogs, { props: { containerId: 1 } });
		expect(screen.getByText('Last 50')).toBeInTheDocument();
		expect(screen.getByText('Last 100')).toBeInTheDocument();
		expect(screen.getByText('Last 500')).toBeInTheDocument();
		expect(screen.getByText('Last 1000')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// ContainerStats
// ──────────────────────────────────────────────────────────
describe('ContainerStats Component', () => {
	let ContainerStats: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/containers/ContainerStats.svelte');
		ContainerStats = mod.default;
	});

	it('should render loading message initially', () => {
		render(ContainerStats, { props: { containerId: 1 } });
		expect(screen.getByText('Loading stats...')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// ContainerDetail
// ──────────────────────────────────────────────────────────
describe('ContainerDetail Component', () => {
	let ContainerDetail: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/containers/ContainerDetail.svelte');
		ContainerDetail = mod.default;
	});

	const baseContainer = {
		id: 1,
		name: 'Detail Container',
		description: 'Some description',
		containerType: 'docker',
		image: 'ubuntu:22.04',
		icon: '',
		ports: [],
		volumes: [],
		env: {},
		workingDir: '/app',
		dockerContainerId: 'abc123'
	};

	it('should render container name', () => {
		render(ContainerDetail, {
			props: { container: baseContainer, onClose: vi.fn() }
		});
		expect(screen.getByText('Detail Container')).toBeInTheDocument();
	});

	it('should render description', () => {
		render(ContainerDetail, {
			props: { container: baseContainer, onClose: vi.fn() }
		});
		expect(screen.getByText('Some description')).toBeInTheDocument();
	});

	it('should render tab buttons', () => {
		render(ContainerDetail, {
			props: { container: baseContainer, onClose: vi.fn() }
		});
		expect(screen.getByText('Overview')).toBeInTheDocument();
		expect(screen.getByText('Logs')).toBeInTheDocument();
		expect(screen.getByText('Stats')).toBeInTheDocument();
		expect(screen.getByText('Exec')).toBeInTheDocument();
	});

	it('should show overview tab content by default', () => {
		render(ContainerDetail, {
			props: { container: baseContainer, onClose: vi.fn() }
		});
		expect(screen.getByText('Type:')).toBeInTheDocument();
		expect(screen.getByText('Image:')).toBeInTheDocument();
	});

	it('should render close button', () => {
		render(ContainerDetail, {
			props: { container: baseContainer, onClose: vi.fn() }
		});
		expect(screen.getByLabelText('Close')).toBeInTheDocument();
	});

	it('should render without description when absent', () => {
		const noDesc = { ...baseContainer, description: '' };
		render(ContainerDetail, {
			props: { container: noDesc, onClose: vi.fn() }
		});
		expect(screen.queryByText('Some description')).not.toBeInTheDocument();
	});

	it('should display ports when present', () => {
		const withPorts = {
			...baseContainer,
			ports: [{ hostPort: 8080, containerPort: 80, protocol: 'tcp' }]
		};
		render(ContainerDetail, {
			props: { container: withPorts, onClose: vi.fn() }
		});
		expect(screen.getByText('Ports')).toBeInTheDocument();
		expect(screen.getByText('8080:80/tcp')).toBeInTheDocument();
	});

	it('should display env vars when present', () => {
		const withEnv = {
			...baseContainer,
			env: { NODE_ENV: 'production' }
		};
		render(ContainerDetail, {
			props: { container: withEnv, onClose: vi.fn() }
		});
		expect(screen.getByText('Environment')).toBeInTheDocument();
		expect(screen.getByText('NODE_ENV')).toBeInTheDocument();
		expect(screen.getByText('production')).toBeInTheDocument();
	});

	it('should show N/A for missing fields', () => {
		const sparse = {
			...baseContainer,
			image: '',
			workingDir: '',
			dockerContainerId: ''
		};
		render(ContainerDetail, {
			props: { container: sparse, onClose: vi.fn() }
		});
		const naElements = screen.getAllByText('N/A');
		expect(naElements.length).toBeGreaterThanOrEqual(1);
	});
});

// ──────────────────────────────────────────────────────────
// PortMappingEditor
// ──────────────────────────────────────────────────────────
describe('PortMappingEditor Component', () => {
	let PortMappingEditor: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/containers/PortMappingEditor.svelte');
		PortMappingEditor = mod.default;
	});

	it('should render Port Mappings label', () => {
		render(PortMappingEditor, { props: { ports: [] } });
		expect(screen.getByText('Port Mappings')).toBeInTheDocument();
	});

	it('should render Add Port button', () => {
		render(PortMappingEditor, { props: { ports: [] } });
		expect(screen.getByText('Add Port')).toBeInTheDocument();
	});

	it('should render port entries when ports are provided', () => {
		const ports = [{ hostPort: 3000, containerPort: 80, protocol: 'tcp' }];
		render(PortMappingEditor, { props: { ports } });
		expect(screen.getByText('TCP')).toBeInTheDocument();
	});

	it('should not render port entries when empty', () => {
		render(PortMappingEditor, { props: { ports: [] } });
		expect(screen.queryByText('TCP')).not.toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// VolumeMappingEditor
// ──────────────────────────────────────────────────────────
describe('VolumeMappingEditor Component', () => {
	let VolumeMappingEditor: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/containers/VolumeMappingEditor.svelte');
		VolumeMappingEditor = mod.default;
	});

	it('should render Volume Mappings label', () => {
		render(VolumeMappingEditor, { props: { volumes: [] } });
		expect(screen.getByText('Volume Mappings')).toBeInTheDocument();
	});

	it('should render Add Volume button', () => {
		render(VolumeMappingEditor, { props: { volumes: [] } });
		expect(screen.getByText('Add Volume')).toBeInTheDocument();
	});

	it('should render volume entries when provided', () => {
		const volumes = [{ hostPath: '/data', containerPath: '/mnt', readOnly: false }];
		render(VolumeMappingEditor, { props: { volumes } });
		expect(screen.getByText('RO')).toBeInTheDocument();
	});

	it('should not render volume entries when empty', () => {
		render(VolumeMappingEditor, { props: { volumes: [] } });
		expect(screen.queryByText('RO')).not.toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// TemplateCard
// ──────────────────────────────────────────────────────────
describe('TemplateCard Component', () => {
	let TemplateCard: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/containers/TemplateCard.svelte');
		TemplateCard = mod.default;
	});

	const mockTemplate = {
		id: 't1',
		name: 'Node.js Dev',
		description: 'A Node development environment',
		category: 'web',
		image: 'node:20',
		icon: '\u{1F310}'
	};

	it('should render template name', () => {
		render(TemplateCard, { props: { template: mockTemplate, onUse: vi.fn() } });
		expect(screen.getByText('Node.js Dev')).toBeInTheDocument();
	});

	it('should render template description', () => {
		render(TemplateCard, { props: { template: mockTemplate, onUse: vi.fn() } });
		expect(screen.getByText('A Node development environment')).toBeInTheDocument();
	});

	it('should render category and image', () => {
		render(TemplateCard, { props: { template: mockTemplate, onUse: vi.fn() } });
		expect(screen.getByText('web')).toBeInTheDocument();
		expect(screen.getByText('node:20')).toBeInTheDocument();
	});

	it('should render clickable card', () => {
		const onUse = vi.fn();
		render(TemplateCard, { props: { template: mockTemplate, onUse } });
		const card = screen.getByText('Node.js Dev').closest('.card');
		expect(card).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// TemplateBrowser
// ──────────────────────────────────────────────────────────
describe('TemplateBrowser Component', () => {
	let TemplateBrowser: any;
	let containerLibrary: any;

	beforeAll(async () => {
		const stores = await import('$lib/stores');
		containerLibrary = (stores as any).containerLibrary;
		const mod = await import('$lib/components/containers/TemplateBrowser.svelte');
		TemplateBrowser = mod.default;
	});

	it('should render All category button when templates exist', () => {
		containerLibrary.templates = [
			{ id: '1', name: 'T1', description: 'Desc', category: 'web', image: 'img', icon: '' }
		];
		render(TemplateBrowser, { props: { onUse: vi.fn() } });
		expect(screen.getByText('All')).toBeInTheDocument();
	});

	it('should render category buttons from templates', () => {
		containerLibrary.templates = [
			{ id: '1', name: 'T1', description: 'Desc', category: 'web', image: 'img', icon: '' },
			{ id: '2', name: 'T2', description: 'Desc', category: 'data', image: 'img', icon: '' }
		];
		render(TemplateBrowser, { props: { onUse: vi.fn() } });
		expect(screen.getByText('Web')).toBeInTheDocument();
		expect(screen.getByText('Data')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// DockerHostForm
// ──────────────────────────────────────────────────────────
describe('DockerHostForm Component', () => {
	let DockerHostForm: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/containers/DockerHostForm.svelte');
		DockerHostForm = mod.default;
	});

	it('should render Name label', () => {
		render(DockerHostForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Name *')).toBeInTheDocument();
	});

	it('should render Host Type select', () => {
		render(DockerHostForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Host Type')).toBeInTheDocument();
		expect(screen.getByText('Local')).toBeInTheDocument();
		expect(screen.getByText('SSH')).toBeInTheDocument();
		expect(screen.getByText('TCP')).toBeInTheDocument();
	});

	it('should render Cancel and Add Host buttons', () => {
		render(DockerHostForm, { props: { onSubmit: vi.fn(), onCancel: vi.fn() } });
		expect(screen.getByText('Cancel')).toBeInTheDocument();
		expect(screen.getByText('Add Host')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// DockerHostList
// ──────────────────────────────────────────────────────────
describe('DockerHostList Component', () => {
	let DockerHostList: any;
	let containerLibrary: any;

	beforeAll(async () => {
		const stores = await import('$lib/stores');
		containerLibrary = (stores as any).containerLibrary;
		const mod = await import('$lib/components/containers/DockerHostList.svelte');
		DockerHostList = mod.default;
	});

	beforeEach(() => {
		containerLibrary.dockerHosts = [];
	});

	it('should render empty state when no hosts', () => {
		render(DockerHostList);
		expect(screen.getByText('No Docker hosts configured')).toBeInTheDocument();
	});

	it('should render host entries', () => {
		containerLibrary.dockerHosts = [
			{ id: 1, name: 'Local Docker', hostType: 'local', connectionUri: '', isDefault: true }
		];
		render(DockerHostList);
		expect(screen.getByText('Local Docker')).toBeInTheDocument();
		expect(screen.getByText('Default')).toBeInTheDocument();
	});

	it('should render test connection button', () => {
		containerLibrary.dockerHosts = [
			{ id: 1, name: 'Local', hostType: 'local', connectionUri: '', isDefault: false }
		];
		render(DockerHostList);
		expect(screen.getByLabelText('Test connection')).toBeInTheDocument();
	});

	it('should not render delete button for host id 1', () => {
		containerLibrary.dockerHosts = [
			{ id: 1, name: 'Local', hostType: 'local', connectionUri: '', isDefault: false }
		];
		render(DockerHostList);
		expect(screen.queryByLabelText('Delete host')).not.toBeInTheDocument();
	});

	it('should render delete button for hosts with id != 1', () => {
		containerLibrary.dockerHosts = [
			{ id: 2, name: 'Remote', hostType: 'ssh', connectionUri: 'ssh://user@host', isDefault: false }
		];
		render(DockerHostList);
		expect(screen.getByLabelText('Delete host')).toBeInTheDocument();
	});

	it('should show host type and connection URI', () => {
		containerLibrary.dockerHosts = [
			{ id: 2, name: 'Remote', hostType: 'ssh', connectionUri: 'ssh://user@host', isDefault: false }
		];
		render(DockerHostList);
		const text = document.body.textContent;
		expect(text).toContain('ssh');
		expect(text).toContain('ssh://user@host');
	});
});

// ──────────────────────────────────────────────────────────
// ProjectContainerPanel
// ──────────────────────────────────────────────────────────
describe('ProjectContainerPanel Component', () => {
	let ProjectContainerPanel: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/containers/ProjectContainerPanel.svelte');
		ProjectContainerPanel = mod.default;
	});

	it('should render Containers heading', () => {
		render(ProjectContainerPanel, { props: { projectId: 1 } });
		expect(screen.getByText('Containers')).toBeInTheDocument();
	});

	it('should render Assign button', () => {
		render(ProjectContainerPanel, { props: { projectId: 1 } });
		expect(screen.getByText('Assign')).toBeInTheDocument();
	});

	it('should render empty state for no containers', () => {
		render(ProjectContainerPanel, { props: { projectId: 1 } });
		expect(screen.getByText('No containers assigned')).toBeInTheDocument();
	});
});

// ──────────────────────────────────────────────────────────
// Containers index.ts exports
// ──────────────────────────────────────────────────────────
describe('Containers index.ts exports', () => {
	let containerExports: any;

	beforeAll(async () => {
		containerExports = await import('$lib/components/containers');
	});

	it('should export all container components', () => {
		expect(containerExports.ContainerList).toBeDefined();
		expect(containerExports.ContainerCard).toBeDefined();
		expect(containerExports.ContainerForm).toBeDefined();
		expect(containerExports.ContainerDetail).toBeDefined();
		expect(containerExports.ContainerStatus).toBeDefined();
		expect(containerExports.ContainerActions).toBeDefined();
		expect(containerExports.ContainerLogs).toBeDefined();
		expect(containerExports.ContainerStats).toBeDefined();
		expect(containerExports.PortMappingEditor).toBeDefined();
		expect(containerExports.VolumeMappingEditor).toBeDefined();
		expect(containerExports.TemplateBrowser).toBeDefined();
		expect(containerExports.TemplateCard).toBeDefined();
		expect(containerExports.DockerHostList).toBeDefined();
		expect(containerExports.DockerHostForm).toBeDefined();
		expect(containerExports.ProjectContainerPanel).toBeDefined();
	});
});
