import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte/pure';

vi.mock('$lib/stores', () => ({
	mcpLibrary: {
		mcps: [],
		filteredMcps: [],
		isLoading: false,
		searchQuery: '',
		selectedType: 'all',
		load: vi.fn(),
		getMcpById: vi.fn(),
		updateMcp: vi.fn(),
		setTypeFilter: vi.fn(),
		mcpCount: { total: 0, stdio: 0, sse: 0, http: 0 }
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	},
	repoLibrary: {
		repos: [],
		load: vi.fn(),
		searchQuery: ''
	}
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

describe('McpCard Component', () => {
	let McpCard: any;

	const mockMcp = {
		id: 1,
		name: 'Test MCP',
		type: 'stdio' as const,
		command: 'npx test-mcp',
		args: [],
		env: {},
		description: 'A test MCP server',
		source: 'user' as const,
		isFavorite: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/mcp/McpCard.svelte');
		McpCard = mod.default;
	});

	it('should render MCP name', () => {
		render(McpCard, { props: { mcp: mockMcp } });
		expect(screen.getByText('Test MCP')).toBeInTheDocument();
	});

	it('should show MCP type', () => {
		render(McpCard, { props: { mcp: mockMcp } });
		expect(screen.getByText('stdio')).toBeInTheDocument();
	});

	it('should show description', () => {
		render(McpCard, { props: { mcp: mockMcp } });
		expect(screen.getByText('A test MCP server')).toBeInTheDocument();
	});

	it('should hide actions when showActions is false', () => {
		render(McpCard, { props: { mcp: mockMcp, showActions: false } });
		expect(screen.queryByLabelText(/Actions/)).not.toBeInTheDocument();
	});

	it('should show actions menu when showActions is true (default)', () => {
		render(McpCard, { props: { mcp: mockMcp } });
		expect(screen.getByLabelText(`Actions for ${mockMcp.name}`)).toBeInTheDocument();
	});

	it('should show System badge for system MCP', () => {
		const systemMcp = { ...mockMcp, source: 'system' as const };
		render(McpCard, { props: { mcp: systemMcp } });
		expect(screen.getByText('System')).toBeInTheDocument();
	});

	it('should show Auto badge for auto-detected MCP', () => {
		const autoMcp = { ...mockMcp, source: 'auto-detected' as const };
		render(McpCard, { props: { mcp: autoMcp } });
		expect(screen.getByText('Auto')).toBeInTheDocument();
	});

	it('should not show System or Auto badge for user MCP', () => {
		render(McpCard, { props: { mcp: mockMcp } });
		expect(screen.queryByText('System')).not.toBeInTheDocument();
		expect(screen.queryByText('Auto')).not.toBeInTheDocument();
	});

	it('should show command for stdio type', () => {
		render(McpCard, { props: { mcp: mockMcp } });
		expect(screen.getByText('npx test-mcp')).toBeInTheDocument();
	});

	it('should show URL hostname for SSE type', () => {
		const sseMcp = { ...mockMcp, type: 'sse' as const, command: undefined, url: 'https://api.example.com/sse' };
		render(McpCard, { props: { mcp: sseMcp } });
		expect(screen.getByText('api.example.com')).toBeInTheDocument();
	});

	it('should show URL hostname for HTTP type', () => {
		const httpMcp = { ...mockMcp, type: 'http' as const, command: undefined, url: 'https://api.example.com/v1' };
		render(McpCard, { props: { mcp: httpMcp } });
		expect(screen.getByText('api.example.com')).toBeInTheDocument();
	});

	it('should not show description when not provided', () => {
		const mcpNoDesc = { ...mockMcp, description: undefined };
		render(McpCard, { props: { mcp: mcpNoDesc } });
		expect(screen.queryByText('A test MCP server')).not.toBeInTheDocument();
	});

	it('should show FavoriteButton when onFavoriteToggle provided', () => {
		render(McpCard, { props: { mcp: mockMcp, onFavoriteToggle: vi.fn() } });
		expect(screen.getByLabelText(`Add ${mockMcp.name} to favorites`)).toBeInTheDocument();
	});

	it('should not show FavoriteButton when onFavoriteToggle not provided', () => {
		render(McpCard, { props: { mcp: mockMcp } });
		expect(screen.queryByLabelText(`Add ${mockMcp.name} to favorites`)).not.toBeInTheDocument();
	});

	it('should show Gateway badge when showGatewayToggle and isInGateway', () => {
		render(McpCard, {
			props: { mcp: mockMcp, showGatewayToggle: true, isInGateway: true }
		});
		expect(screen.getByText('Gateway')).toBeInTheDocument();
	});

	it('should show "In Gateway" text when in gateway', () => {
		render(McpCard, {
			props: { mcp: mockMcp, showGatewayToggle: true, isInGateway: true }
		});
		expect(screen.getByText('In Gateway')).toBeInTheDocument();
	});

	it('should show "Add to Gateway" text when not in gateway', () => {
		render(McpCard, {
			props: { mcp: mockMcp, showGatewayToggle: true, isInGateway: false }
		});
		expect(screen.getByText('Add to Gateway')).toBeInTheDocument();
	});

	it('should not show gateway toggle when showGatewayToggle is false', () => {
		render(McpCard, { props: { mcp: mockMcp, showGatewayToggle: false } });
		expect(screen.queryByText('Add to Gateway')).not.toBeInTheDocument();
		expect(screen.queryByText('In Gateway')).not.toBeInTheDocument();
	});

	it('should show sse type badge', () => {
		const sseMcp = { ...mockMcp, type: 'sse' as const };
		render(McpCard, { props: { mcp: sseMcp } });
		expect(screen.getByText('sse')).toBeInTheDocument();
	});

	it('should show http type badge', () => {
		const httpMcp = { ...mockMcp, type: 'http' as const };
		render(McpCard, { props: { mcp: httpMcp } });
		expect(screen.getByText('http')).toBeInTheDocument();
	});
});

describe('McpTypeSelector Component', () => {
	let McpTypeSelector: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/mcp/McpTypeSelector.svelte');
		McpTypeSelector = mod.default;
	});

	it('should render all three types', () => {
		render(McpTypeSelector, { props: { value: 'stdio' } });
		expect(screen.getByText('Standard I/O')).toBeInTheDocument();
		expect(screen.getByText('Server-Sent Events')).toBeInTheDocument();
		expect(screen.getByText('HTTP/REST')).toBeInTheDocument();
	});

	it('should show descriptions for each type', () => {
		render(McpTypeSelector, { props: { value: 'stdio' } });
		expect(screen.getByText('Local command-line tool (npx, python, etc.)')).toBeInTheDocument();
		expect(screen.getByText('Cloud service with SSE endpoint')).toBeInTheDocument();
		expect(screen.getByText('REST API with token authentication')).toBeInTheDocument();
	});

	it('should show Connection Type label', () => {
		render(McpTypeSelector, { props: { value: 'stdio' } });
		expect(screen.getByText('Connection Type')).toBeInTheDocument();
	});
});

describe('McpLibrary Component', () => {
	let McpLibrary: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/mcp/McpLibrary.svelte');
		McpLibrary = mod.default;
	});

	it('should render library component', () => {
		render(McpLibrary, { props: {} });
		expect(document.body).toBeTruthy();
	});

	it('should show search bar', () => {
		render(McpLibrary, { props: {} });
		expect(screen.getByPlaceholderText('Search MCPs...')).toBeInTheDocument();
	});

	it('should show type filter buttons', () => {
		render(McpLibrary, { props: {} });
		expect(screen.getByText('All')).toBeInTheDocument();
		// "stdio" appears as both a label and a count area
		expect(screen.getAllByText('stdio').length).toBeGreaterThan(0);
		expect(screen.getByText('SSE')).toBeInTheDocument();
		expect(screen.getByText('HTTP')).toBeInTheDocument();
	});

	it('should show empty state when no MCPs', () => {
		render(McpLibrary, { props: {} });
		expect(screen.getByText('No MCPs in library')).toBeInTheDocument();
	});

	it('should show empty state description', () => {
		render(McpLibrary, { props: {} });
		expect(screen.getByText('Add your first MCP to get started')).toBeInTheDocument();
	});
});

describe('McpTestModal Component', () => {
	let McpTestModal: any;

	const mockMcp = {
		id: 1,
		name: 'Test Server',
		type: 'stdio' as const,
		command: 'npx test-server',
		args: [],
		env: {},
		source: 'user' as const,
		isFavorite: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/mcp/McpTestModal.svelte');
		McpTestModal = mod.default;
	});

	it('should render modal with MCP name', () => {
		render(McpTestModal, { props: { mcp: mockMcp, onClose: vi.fn() } });
		expect(screen.getByText('Test MCP: Test Server')).toBeInTheDocument();
	});

	it('should show command for stdio MCP', () => {
		render(McpTestModal, { props: { mcp: mockMcp, onClose: vi.fn() } });
		expect(screen.getByText('npx test-server')).toBeInTheDocument();
	});

	it('should show URL for SSE MCP', () => {
		const sseMcp = { ...mockMcp, type: 'sse' as const, command: undefined, url: 'https://api.example.com/sse' };
		render(McpTestModal, { props: { mcp: sseMcp, onClose: vi.fn() } });
		expect(screen.getByText('https://api.example.com/sse')).toBeInTheDocument();
	});

	it('should show loading state initially', () => {
		render(McpTestModal, { props: { mcp: mockMcp, onClose: vi.fn() } });
		expect(screen.getByText('Testing connection...')).toBeInTheDocument();
		expect(screen.getByText('This may take a few seconds')).toBeInTheDocument();
	});

	it('should show Re-run Test button (disabled during loading)', () => {
		render(McpTestModal, { props: { mcp: mockMcp, onClose: vi.fn() } });
		expect(screen.getByText('Testing...')).toBeInTheDocument();
	});

	it('should show Close button', () => {
		render(McpTestModal, { props: { mcp: mockMcp, onClose: vi.fn() } });
		expect(screen.getByText('Close')).toBeInTheDocument();
	});
});

describe('McpExecutionModal Component', () => {
	let McpExecutionModal: any;

	const mockMcp = {
		id: 1,
		name: 'Exec Server',
		type: 'stdio' as const,
		command: 'npx exec-server',
		args: [],
		env: {},
		source: 'user' as const,
		isFavorite: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/mcp/McpExecutionModal.svelte');
		McpExecutionModal = mod.default;
	});

	it('should render modal with MCP name', () => {
		render(McpExecutionModal, { props: { mcp: mockMcp, onClose: vi.fn() } });
		expect(screen.getByText('Execute: Exec Server')).toBeInTheDocument();
	});

	it('should show connecting state initially', () => {
		render(McpExecutionModal, { props: { mcp: mockMcp, onClose: vi.fn() } });
		expect(screen.getByText('Starting session...')).toBeInTheDocument();
		expect(screen.getByText('This may take a few seconds')).toBeInTheDocument();
	});

	it('should show Connecting... status', () => {
		render(McpExecutionModal, { props: { mcp: mockMcp, onClose: vi.fn() } });
		expect(screen.getByText('Connecting...')).toBeInTheDocument();
	});

	it('should show Close button', () => {
		render(McpExecutionModal, { props: { mcp: mockMcp, onClose: vi.fn() } });
		expect(screen.getByText('Close')).toBeInTheDocument();
	});
});

describe('JsonSchemaForm Component', () => {
	let JsonSchemaForm: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/mcp/JsonSchemaForm.svelte');
		JsonSchemaForm = mod.default;
	});

	it('should show no parameters message for null schema', () => {
		render(JsonSchemaForm, {
			props: { schema: null, value: {}, onChange: vi.fn() }
		});
		expect(screen.getByText('This tool takes no parameters')).toBeInTheDocument();
	});

	it('should show no parameters message for empty schema', () => {
		render(JsonSchemaForm, {
			props: { schema: {}, value: {}, onChange: vi.fn() }
		});
		expect(screen.getByText('This tool takes no parameters')).toBeInTheDocument();
	});

	it('should render string fields from schema properties', () => {
		const schema = {
			type: 'object',
			properties: {
				name: { type: 'string', description: 'The name' }
			}
		};
		render(JsonSchemaForm, {
			props: { schema, value: {}, onChange: vi.fn() }
		});
		expect(screen.getByText('name')).toBeInTheDocument();
		expect(screen.getByText('The name')).toBeInTheDocument();
	});

	it('should render number fields', () => {
		const schema = {
			type: 'object',
			properties: {
				count: { type: 'number', description: 'A count value' }
			}
		};
		render(JsonSchemaForm, {
			props: { schema, value: {}, onChange: vi.fn() }
		});
		expect(screen.getByText('count')).toBeInTheDocument();
	});

	it('should render boolean fields as checkboxes', () => {
		const schema = {
			type: 'object',
			properties: {
				enabled: { type: 'boolean', description: 'Enable feature' }
			}
		};
		render(JsonSchemaForm, {
			props: { schema, value: {}, onChange: vi.fn() }
		});
		expect(screen.getByText('enabled')).toBeInTheDocument();
		expect(screen.getByText('Enable feature')).toBeInTheDocument();
		const checkbox = document.querySelector('input[type="checkbox"]');
		expect(checkbox).toBeInTheDocument();
	});

	it('should render enum fields as selects', () => {
		const schema = {
			type: 'object',
			properties: {
				color: { type: 'string', enum: ['red', 'green', 'blue'], description: 'Choose a color' }
			}
		};
		render(JsonSchemaForm, {
			props: { schema, value: {}, onChange: vi.fn() }
		});
		expect(screen.getByText('color')).toBeInTheDocument();
		expect(screen.getByText('Select...')).toBeInTheDocument();
	});

	it('should render array fields with Add button', () => {
		const schema = {
			type: 'object',
			properties: {
				tags: { type: 'array', items: { type: 'string' }, description: 'Tag list' }
			}
		};
		render(JsonSchemaForm, {
			props: { schema, value: {}, onChange: vi.fn() }
		});
		expect(screen.getByText('tags')).toBeInTheDocument();
		expect(screen.getByText('Add')).toBeInTheDocument();
	});

	it('should render nested object fields', () => {
		const schema = {
			type: 'object',
			properties: {
				config: {
					type: 'object',
					properties: {
						host: { type: 'string' }
					}
				}
			}
		};
		render(JsonSchemaForm, {
			props: { schema, value: {}, onChange: vi.fn() }
		});
		expect(screen.getByText('config')).toBeInTheDocument();
	});

	it('should show required indicator for required fields', () => {
		const schema = {
			type: 'object',
			required: ['name'],
			properties: {
				name: { type: 'string' }
			}
		};
		render(JsonSchemaForm, {
			props: { schema, value: {}, onChange: vi.fn() }
		});
		// Required fields show a red asterisk
		const asterisks = document.querySelectorAll('.text-red-500');
		expect(asterisks.length).toBeGreaterThan(0);
	});

	it('should call onChange when input value changes', async () => {
		const onChange = vi.fn();
		const schema = {
			type: 'object',
			properties: {
				name: { type: 'string' }
			}
		};
		render(JsonSchemaForm, {
			props: { schema, value: {}, onChange }
		});
		const input = document.querySelector('input[type="text"]')!;
		await fireEvent.input(input, { target: { value: 'hello' } });
		expect(onChange).toHaveBeenCalledWith({ name: 'hello' });
	});

	it('should call onChange when checkbox changes', async () => {
		const onChange = vi.fn();
		const schema = {
			type: 'object',
			properties: {
				enabled: { type: 'boolean' }
			}
		};
		render(JsonSchemaForm, {
			props: { schema, value: {}, onChange }
		});
		const checkbox = document.querySelector('input[type="checkbox"]')!;
		await fireEvent.change(checkbox, { target: { checked: true } });
		expect(onChange).toHaveBeenCalledWith({ enabled: true });
	});

	it('should show JSON textarea fallback for non-object schema', () => {
		const schema = {
			type: 'string'
		};
		render(JsonSchemaForm, {
			props: { schema, value: {}, onChange: vi.fn() }
		});
		expect(screen.getByText('Arguments (JSON)')).toBeInTheDocument();
	});

	it('should disable fields when disabled prop is true', () => {
		const schema = {
			type: 'object',
			properties: {
				name: { type: 'string' }
			}
		};
		render(JsonSchemaForm, {
			props: { schema, value: {}, onChange: vi.fn(), disabled: true }
		});
		const input = document.querySelector('input[type="text"]') as HTMLInputElement;
		expect(input.disabled).toBe(true);
	});

	it('should render existing values', () => {
		const schema = {
			type: 'object',
			properties: {
				name: { type: 'string' }
			}
		};
		render(JsonSchemaForm, {
			props: { schema, value: { name: 'existing' }, onChange: vi.fn() }
		});
		const input = document.querySelector('input[type="text"]') as HTMLInputElement;
		expect(input.value).toBe('existing');
	});

	it('should add array items when Add button clicked', async () => {
		const onChange = vi.fn();
		const schema = {
			type: 'object',
			properties: {
				items: { type: 'array', items: { type: 'string' } }
			}
		};
		render(JsonSchemaForm, {
			props: { schema, value: {}, onChange }
		});
		await fireEvent.click(screen.getByText('Add'));
		expect(onChange).toHaveBeenCalledWith({ items: [''] });
	});
});

describe('MCP index.ts exports', () => {
	let mcpExports: any;

	beforeAll(async () => {
		mcpExports = await import('$lib/components/mcp');
	});

	it('should export all MCP components', () => {
		expect(mcpExports.McpCard).toBeDefined();
		expect(mcpExports.McpForm).toBeDefined();
		expect(mcpExports.McpLibrary).toBeDefined();
		expect(mcpExports.McpTestModal).toBeDefined();
		expect(mcpExports.McpTypeSelector).toBeDefined();
	});
});
