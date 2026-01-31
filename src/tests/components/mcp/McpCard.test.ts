import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import McpCard from '$lib/components/mcp/McpCard.svelte';
import type { Mcp } from '$lib/types';

describe('McpCard', () => {
	const createMockMcp = (overrides: Partial<Mcp> = {}): Mcp => ({
		id: 1,
		name: 'Test MCP',
		type: 'stdio',
		command: 'npx test-mcp',
		source: 'user',
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01',
		...overrides
	});

	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render MCP name', () => {
			render(McpCard, { props: { mcp: createMockMcp({ name: 'My Custom MCP' }) } });

			expect(screen.getByText('My Custom MCP')).toBeInTheDocument();
		});

		it('should render MCP description when provided', () => {
			render(McpCard, {
				props: { mcp: createMockMcp({ description: 'A test description' }) }
			});

			expect(screen.getByText('A test description')).toBeInTheDocument();
		});

		it('should render MCP type badge', () => {
			render(McpCard, { props: { mcp: createMockMcp({ type: 'stdio' }) } });

			expect(screen.getByText('stdio')).toBeInTheDocument();
		});

		it('should render command for stdio type', () => {
			render(McpCard, {
				props: { mcp: createMockMcp({ type: 'stdio', command: 'npx test-server' }) }
			});

			expect(screen.getByText('npx test-server')).toBeInTheDocument();
		});

		it('should render hostname for HTTP type', () => {
			render(McpCard, {
				props: {
					mcp: createMockMcp({
						type: 'http',
						command: undefined,
						url: 'https://api.example.com/mcp'
					})
				}
			});

			expect(screen.getByText('api.example.com')).toBeInTheDocument();
		});

		it('should render hostname for SSE type', () => {
			render(McpCard, {
				props: {
					mcp: createMockMcp({
						type: 'sse',
						command: undefined,
						url: 'https://stream.example.com/sse'
					})
				}
			});

			expect(screen.getByText('stream.example.com')).toBeInTheDocument();
		});
	});

	describe('type styling', () => {
		it('should apply stdio type colors', () => {
			const { container } = render(McpCard, {
				props: { mcp: createMockMcp({ type: 'stdio' }) }
			});

			const iconContainer = container.querySelector('.bg-purple-100');
			expect(iconContainer).toBeInTheDocument();
		});

		it('should apply SSE type colors', () => {
			const { container } = render(McpCard, {
				props: { mcp: createMockMcp({ type: 'sse', url: 'https://example.com' }) }
			});

			const iconContainer = container.querySelector('.bg-green-100');
			expect(iconContainer).toBeInTheDocument();
		});

		it('should apply HTTP type colors', () => {
			const { container } = render(McpCard, {
				props: { mcp: createMockMcp({ type: 'http', url: 'https://example.com' }) }
			});

			const iconContainer = container.querySelector('.bg-blue-100');
			expect(iconContainer).toBeInTheDocument();
		});
	});

	describe('source badges', () => {
		it('should show System badge for system MCPs', () => {
			render(McpCard, {
				props: { mcp: createMockMcp({ source: 'system' }) }
			});

			expect(screen.getByText('System')).toBeInTheDocument();
		});

		it('should show Auto badge for auto-detected MCPs', () => {
			render(McpCard, {
				props: { mcp: createMockMcp({ source: 'auto-detected' }) }
			});

			expect(screen.getByText('Auto')).toBeInTheDocument();
		});

		it('should not show badge for user MCPs', () => {
			render(McpCard, {
				props: { mcp: createMockMcp({ source: 'user' }) }
			});

			expect(screen.queryByText('System')).not.toBeInTheDocument();
			expect(screen.queryByText('Auto')).not.toBeInTheDocument();
		});
	});

	describe('actions menu', () => {
		it('should show menu button when showActions is true', () => {
			render(McpCard, {
				props: { mcp: createMockMcp(), showActions: true }
			});

			// Menu button exists
			const buttons = screen.getAllByRole('button');
			expect(buttons.length).toBeGreaterThan(0);
		});

		it('should not show menu button when showActions is false', () => {
			render(McpCard, {
				props: { mcp: createMockMcp(), showActions: false }
			});

			// No buttons
			expect(screen.queryByRole('button')).not.toBeInTheDocument();
		});

		it('should toggle menu on button click', async () => {
			const onEdit = vi.fn();
			render(McpCard, {
				props: { mcp: createMockMcp(), onEdit }
			});

			// Click menu button
			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);

			// Menu should show Edit option
			expect(screen.getByText('Edit')).toBeInTheDocument();
		});

		it('should call onEdit when Edit is clicked', async () => {
			const onEdit = vi.fn();
			const mcp = createMockMcp();
			render(McpCard, { props: { mcp, onEdit } });

			// Open menu
			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);

			// Click Edit
			await fireEvent.click(screen.getByText('Edit'));

			expect(onEdit).toHaveBeenCalledWith(mcp);
		});

		it('should call onDelete when Delete is clicked', async () => {
			const onDelete = vi.fn();
			const mcp = createMockMcp();
			render(McpCard, { props: { mcp, onDelete } });

			// Open menu
			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);

			// Click Delete
			await fireEvent.click(screen.getByText('Delete'));

			expect(onDelete).toHaveBeenCalledWith(mcp);
		});

		it('should call onDuplicate when Duplicate is clicked', async () => {
			const onDuplicate = vi.fn();
			const mcp = createMockMcp();
			render(McpCard, { props: { mcp, onDuplicate } });

			// Open menu
			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);

			// Click Duplicate
			await fireEvent.click(screen.getByText('Duplicate'));

			expect(onDuplicate).toHaveBeenCalledWith(mcp);
		});

		it('should call onTest when Test is clicked', async () => {
			const onTest = vi.fn();
			const mcp = createMockMcp();
			render(McpCard, { props: { mcp, onTest } });

			// Open menu
			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);

			// Click Test
			await fireEvent.click(screen.getByText('Test'));

			expect(onTest).toHaveBeenCalledWith(mcp);
		});

		it('should close menu after action', async () => {
			const onEdit = vi.fn();
			render(McpCard, { props: { mcp: createMockMcp(), onEdit } });

			// Open menu
			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);

			// Click Edit
			await fireEvent.click(screen.getByText('Edit'));

			// Menu should be closed (Edit option should not be visible)
			expect(screen.queryByText('Edit')).not.toBeInTheDocument();
		});
	});

	describe('system MCP restrictions', () => {
		it('should not show Edit option for system MCPs', async () => {
			const onEdit = vi.fn();
			render(McpCard, {
				props: { mcp: createMockMcp({ source: 'system' }), onEdit }
			});

			// Open menu
			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);

			// Edit should not be visible
			expect(screen.queryByText('Edit')).not.toBeInTheDocument();
		});

		it('should not show Delete option for system MCPs', async () => {
			const onDelete = vi.fn();
			render(McpCard, {
				props: { mcp: createMockMcp({ source: 'system' }), onDelete }
			});

			// Open menu
			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);

			// Delete should not be visible
			expect(screen.queryByText('Delete')).not.toBeInTheDocument();
		});

		it('should not show Duplicate option for system MCPs', async () => {
			const onDuplicate = vi.fn();
			render(McpCard, {
				props: { mcp: createMockMcp({ source: 'system' }), onDuplicate }
			});

			// Open menu
			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);

			// Duplicate should not be visible
			expect(screen.queryByText('Duplicate')).not.toBeInTheDocument();
		});

		it('should still show Test option for system MCPs', async () => {
			const onTest = vi.fn();
			render(McpCard, {
				props: { mcp: createMockMcp({ source: 'system' }), onTest }
			});

			// Open menu
			const menuButton = screen.getByRole('button');
			await fireEvent.click(menuButton);

			// Test should still be visible
			expect(screen.getByText('Test')).toBeInTheDocument();
		});
	});

	describe('gateway toggle', () => {
		it('should not show gateway toggle by default', () => {
			render(McpCard, { props: { mcp: createMockMcp() } });

			expect(screen.queryByText(/Gateway/i)).not.toBeInTheDocument();
		});

		it('should show gateway toggle when enabled', () => {
			render(McpCard, {
				props: {
					mcp: createMockMcp(),
					showGatewayToggle: true,
					isInGateway: false
				}
			});

			expect(screen.getByText('Add to Gateway')).toBeInTheDocument();
		});

		it('should show in gateway state when isInGateway is true', () => {
			render(McpCard, {
				props: {
					mcp: createMockMcp(),
					showGatewayToggle: true,
					isInGateway: true
				}
			});

			expect(screen.getByText('In Gateway')).toBeInTheDocument();
		});

		it('should show Gateway badge when in gateway', () => {
			render(McpCard, {
				props: {
					mcp: createMockMcp(),
					showGatewayToggle: true,
					isInGateway: true
				}
			});

			expect(screen.getByText('Gateway')).toBeInTheDocument();
		});

		it('should call onGatewayToggle when gateway button is clicked', async () => {
			const onGatewayToggle = vi.fn();
			const mcp = createMockMcp();
			render(McpCard, {
				props: {
					mcp,
					showGatewayToggle: true,
					isInGateway: false,
					onGatewayToggle
				}
			});

			await fireEvent.click(screen.getByText('Add to Gateway'));

			expect(onGatewayToggle).toHaveBeenCalledWith(mcp, true);
		});

		it('should toggle off when already in gateway', async () => {
			const onGatewayToggle = vi.fn();
			const mcp = createMockMcp();
			render(McpCard, {
				props: {
					mcp,
					showGatewayToggle: true,
					isInGateway: true,
					onGatewayToggle
				}
			});

			await fireEvent.click(screen.getByText('In Gateway'));

			expect(onGatewayToggle).toHaveBeenCalledWith(mcp, false);
		});
	});
});
