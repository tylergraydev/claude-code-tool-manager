import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import McpForm from '$lib/components/mcp/McpForm.svelte';

describe('McpForm Component', () => {
	const defaultProps = {
		onSubmit: vi.fn(),
		onCancel: vi.fn()
	};

	describe('initial render', () => {
		it('should render name and description fields', () => {
			render(McpForm, { props: defaultProps });

			expect(screen.getByLabelText(/Name/)).toBeInTheDocument();
			expect(screen.getByLabelText(/Description/)).toBeInTheDocument();
		});

		it('should render type selector with all three types', () => {
			render(McpForm, { props: defaultProps });

			expect(screen.getByText('Standard I/O')).toBeInTheDocument();
			expect(screen.getByText('Server-Sent Events')).toBeInTheDocument();
			expect(screen.getByText('HTTP/REST')).toBeInTheDocument();
		});

		it('should show command and args fields for default stdio type', () => {
			render(McpForm, { props: defaultProps });

			expect(screen.getByLabelText(/Command/)).toBeInTheDocument();
			expect(screen.getByLabelText(/Arguments/)).toBeInTheDocument();
		});

		it('should show Create MCP button when no initial values', () => {
			render(McpForm, { props: defaultProps });

			expect(screen.getByText('Create MCP')).toBeInTheDocument();
		});

		it('should show Update MCP button when editing', () => {
			render(McpForm, {
				props: { ...defaultProps, initialValues: { name: 'test-mcp' } }
			});

			expect(screen.getByText('Update MCP')).toBeInTheDocument();
		});

		it('should show Quick Import section', () => {
			render(McpForm, { props: defaultProps });

			expect(screen.getByText('Quick Import')).toBeInTheDocument();
		});

		it('should render cancel button', () => {
			render(McpForm, { props: defaultProps });

			expect(screen.getByText('Cancel')).toBeInTheDocument();
		});
	});

	describe('type switching', () => {
		it('should show URL field when switching to SSE', async () => {
			render(McpForm, { props: defaultProps });

			// Click SSE type
			await fireEvent.click(screen.getByText('Server-Sent Events'));

			expect(screen.getByLabelText(/URL/)).toBeInTheDocument();
			expect(screen.queryByLabelText(/Command/)).not.toBeInTheDocument();
		});

		it('should show URL field when switching to HTTP', async () => {
			render(McpForm, { props: defaultProps });

			await fireEvent.click(screen.getByText('HTTP/REST'));

			expect(screen.getByLabelText(/URL/)).toBeInTheDocument();
			expect(screen.queryByLabelText(/Command/)).not.toBeInTheDocument();
		});

		it('should show Headers section for HTTP type', async () => {
			render(McpForm, { props: defaultProps });

			await fireEvent.click(screen.getByText('HTTP/REST'));

			expect(screen.getByText('Headers')).toBeInTheDocument();
		});

		it('should show command fields when switching back to stdio', async () => {
			render(McpForm, { props: defaultProps });

			// Switch to SSE then back to stdio
			await fireEvent.click(screen.getByText('Server-Sent Events'));
			await fireEvent.click(screen.getByText('Standard I/O'));

			expect(screen.getByLabelText(/Command/)).toBeInTheDocument();
			expect(screen.getByLabelText(/Arguments/)).toBeInTheDocument();
		});
	});

	describe('validation', () => {
		it('should show error when name is empty', async () => {
			render(McpForm, { props: defaultProps });

			await fireEvent.click(screen.getByText('Create MCP'));

			expect(screen.getByText('Name is required')).toBeInTheDocument();
		});

		it('should show error for invalid name characters', async () => {
			render(McpForm, { props: defaultProps });

			const nameInput = screen.getByLabelText(/Name/);
			await fireEvent.input(nameInput, { target: { value: 'my mcp name' } });

			// Also fill command to bypass that validation
			const commandInput = screen.getByLabelText(/Command/);
			await fireEvent.input(commandInput, { target: { value: 'npx' } });

			await fireEvent.click(screen.getByText('Create MCP'));

			expect(screen.getByText(/Name can only contain/)).toBeInTheDocument();
		});

		it('should show error when stdio command is empty', async () => {
			render(McpForm, { props: defaultProps });

			const nameInput = screen.getByLabelText(/Name/);
			await fireEvent.input(nameInput, { target: { value: 'test-mcp' } });

			await fireEvent.click(screen.getByText('Create MCP'));

			expect(screen.getByText('Command is required')).toBeInTheDocument();
		});

		it('should show error when SSE URL is empty', async () => {
			render(McpForm, { props: defaultProps });

			const nameInput = screen.getByLabelText(/Name/);
			await fireEvent.input(nameInput, { target: { value: 'test-mcp' } });

			await fireEvent.click(screen.getByText('Server-Sent Events'));
			await fireEvent.click(screen.getByText('Create MCP'));

			expect(screen.getByText('URL is required')).toBeInTheDocument();
		});

		it('should show error for invalid URL format', async () => {
			render(McpForm, {
				props: {
					...defaultProps,
					initialValues: { type: 'sse', name: 'test-mcp', url: 'not-a-url' }
				}
			});

			const form = document.querySelector('form')!;
			await fireEvent.submit(form);

			expect(screen.getByText('Invalid URL format')).toBeInTheDocument();
		});

		it('should not call onSubmit when validation fails', async () => {
			const onSubmit = vi.fn();
			render(McpForm, { props: { ...defaultProps, onSubmit } });

			await fireEvent.click(screen.getByText('Create MCP'));

			expect(onSubmit).not.toHaveBeenCalled();
		});
	});

	describe('submission', () => {
		it('should call onSubmit with correct stdio data', async () => {
			const onSubmit = vi.fn();
			render(McpForm, { props: { ...defaultProps, onSubmit } });

			const nameInput = screen.getByLabelText(/Name/);
			await fireEvent.input(nameInput, { target: { value: 'test-mcp' } });

			const commandInput = screen.getByLabelText(/Command/);
			await fireEvent.input(commandInput, { target: { value: 'npx' } });

			const argsInput = screen.getByLabelText(/Arguments/);
			await fireEvent.input(argsInput, { target: { value: '-y @test/server' } });

			await fireEvent.click(screen.getByText('Create MCP'));

			expect(onSubmit).toHaveBeenCalledWith(
				expect.objectContaining({
					name: 'test-mcp',
					type: 'stdio',
					command: 'npx',
					args: ['-y', '@test/server']
				})
			);
		});

		it('should call onSubmit with correct SSE data', async () => {
			const onSubmit = vi.fn();
			render(McpForm, { props: { ...defaultProps, onSubmit } });

			const nameInput = screen.getByLabelText(/Name/);
			await fireEvent.input(nameInput, { target: { value: 'test-sse' } });

			await fireEvent.click(screen.getByText('Server-Sent Events'));

			const urlInput = screen.getByLabelText(/URL/);
			await fireEvent.input(urlInput, { target: { value: 'https://example.com/sse' } });

			await fireEvent.click(screen.getByText('Create MCP'));

			expect(onSubmit).toHaveBeenCalledWith(
				expect.objectContaining({
					name: 'test-sse',
					type: 'sse',
					url: 'https://example.com/sse'
				})
			);
		});

		it('should call onCancel when cancel button clicked', async () => {
			const onCancel = vi.fn();
			render(McpForm, { props: { ...defaultProps, onCancel } });

			await fireEvent.click(screen.getByText('Cancel'));

			expect(onCancel).toHaveBeenCalledOnce();
		});
	});

	describe('initial values', () => {
		it('should populate fields from initial values', () => {
			render(McpForm, {
				props: {
					...defaultProps,
					initialValues: {
						name: 'my-mcp',
						description: 'A test MCP',
						type: 'stdio',
						command: 'node',
						args: ['server.js', '--port', '3000']
					}
				}
			});

			expect(screen.getByDisplayValue('my-mcp')).toBeInTheDocument();
			expect(screen.getByDisplayValue('A test MCP')).toBeInTheDocument();
			expect(screen.getByDisplayValue('node')).toBeInTheDocument();
			expect(screen.getByDisplayValue('server.js --port 3000')).toBeInTheDocument();
		});

		it('should populate URL fields for SSE initial values', () => {
			render(McpForm, {
				props: {
					...defaultProps,
					initialValues: {
						name: 'my-sse',
						type: 'sse',
						url: 'https://api.example.com/sse'
					}
				}
			});

			expect(screen.getByDisplayValue('my-sse')).toBeInTheDocument();
			expect(screen.getByDisplayValue('https://api.example.com/sse')).toBeInTheDocument();
		});
	});
});
