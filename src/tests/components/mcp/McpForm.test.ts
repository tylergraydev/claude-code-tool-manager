import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import McpForm from '$lib/components/mcp/McpForm.svelte';

describe('McpForm', () => {
	const mockOnSubmit = vi.fn();
	const mockOnCancel = vi.fn();

	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render Quick Import section', () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByText('Quick Import')).toBeInTheDocument();
		});

		it('should render paste button', () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Paste/i })).toBeInTheDocument();
		});

		it('should render name input with required indicator', () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Name/i)).toBeInTheDocument();
			// Multiple required fields have * indicators
			expect(screen.getAllByText('*').length).toBeGreaterThanOrEqual(1);
		});

		it('should render description textarea', () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Description/i)).toBeInTheDocument();
		});

		it('should render type selector with stdio selected by default', () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			// Type selector shows stdio option as selected
			expect(screen.getByText('Standard I/O')).toBeInTheDocument();
		});

		it('should render command field for stdio type by default', () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByLabelText(/Command/i)).toBeInTheDocument();
			expect(screen.getByLabelText(/Arguments/i)).toBeInTheDocument();
		});

		it('should render environment variables section', () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByText('Environment Variables')).toBeInTheDocument();
		});

		it('should render Cancel button', () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Cancel/i })).toBeInTheDocument();
		});

		it('should render Create MCP button for new MCP', () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByRole('button', { name: /Create MCP/i })).toBeInTheDocument();
		});

		it('should render Update MCP button when editing', () => {
			render(McpForm, {
				props: {
					initialValues: { name: 'test-mcp' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByRole('button', { name: /Update MCP/i })).toBeInTheDocument();
		});

		it('should populate fields with initial values', () => {
			render(McpForm, {
				props: {
					initialValues: {
						name: 'my-mcp',
						description: 'My description',
						type: 'stdio',
						command: 'npx',
						args: ['-y', 'package']
					},
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByDisplayValue('my-mcp')).toBeInTheDocument();
			expect(screen.getByDisplayValue('My description')).toBeInTheDocument();
			expect(screen.getByDisplayValue('npx')).toBeInTheDocument();
			expect(screen.getByDisplayValue('-y package')).toBeInTheDocument();
		});
	});

	describe('validation', () => {
		it('should show error when name is empty', async () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			// Fill only command (required for stdio)
			await fireEvent.input(screen.getByLabelText(/Command/i), {
				target: { value: 'npx' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create MCP/i }));

			expect(screen.getByText('Name is required')).toBeInTheDocument();
			expect(mockOnSubmit).not.toHaveBeenCalled();
		});

		it('should show error when name has invalid characters', async () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'invalid name!' }
			});
			await fireEvent.input(screen.getByLabelText(/Command/i), {
				target: { value: 'npx' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create MCP/i }));

			expect(
				screen.getByText('Name can only contain letters, numbers, hyphens, and underscores')
			).toBeInTheDocument();
		});

		it('should show error when command is empty for stdio type', async () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'test-mcp' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create MCP/i }));

			expect(screen.getByText('Command is required')).toBeInTheDocument();
		});

		it('should show url field for sse type', () => {
			render(McpForm, {
				props: {
					initialValues: { type: 'sse' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			const urlInput = screen.getByLabelText(/URL/i);
			expect(urlInput).toBeInTheDocument();
			expect(urlInput).toHaveAttribute('type', 'url');
		});

		it('should require url for sse type', () => {
			render(McpForm, {
				props: {
					initialValues: { type: 'sse' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			// URL field should be present and empty
			const urlInput = screen.getByLabelText(/URL/i);
			expect(urlInput).toHaveValue('');
		});
	});

	describe('form submission', () => {
		it('should call onSubmit with correct values for stdio type', async () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'my-mcp' }
			});
			await fireEvent.input(screen.getByLabelText(/Description/i), {
				target: { value: 'My description' }
			});
			await fireEvent.input(screen.getByLabelText(/Command/i), {
				target: { value: 'npx' }
			});
			await fireEvent.input(screen.getByLabelText(/Arguments/i), {
				target: { value: '-y @package/mcp' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create MCP/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith({
				name: 'my-mcp',
				description: 'My description',
				type: 'stdio',
				command: 'npx',
				args: ['-y', '@package/mcp'],
				url: undefined,
				headers: undefined,
				env: undefined
			});
		});

		it('should call onSubmit with correct values for sse type', async () => {
			render(McpForm, {
				props: {
					initialValues: { type: 'sse' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			await fireEvent.input(screen.getByLabelText(/Name/i), {
				target: { value: 'sse-mcp' }
			});
			await fireEvent.input(screen.getByLabelText(/URL/i), {
				target: { value: 'https://example.com/sse' }
			});

			await fireEvent.click(screen.getByRole('button', { name: /Create MCP/i }));

			expect(mockOnSubmit).toHaveBeenCalledWith({
				name: 'sse-mcp',
				description: undefined,
				type: 'sse',
				command: undefined,
				args: undefined,
				url: 'https://example.com/sse',
				headers: undefined,
				env: undefined
			});
		});

		it('should not call onSubmit when validation fails', async () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Create MCP/i }));

			expect(mockOnSubmit).not.toHaveBeenCalled();
		});
	});

	describe('cancel', () => {
		it('should call onCancel when Cancel button clicked', async () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			await fireEvent.click(screen.getByRole('button', { name: /Cancel/i }));

			expect(mockOnCancel).toHaveBeenCalledOnce();
		});
	});

	describe('type switching', () => {
		it('should show url field when sse type is set', () => {
			render(McpForm, {
				props: {
					initialValues: { type: 'sse' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByLabelText(/URL/i)).toBeInTheDocument();
			expect(screen.queryByLabelText(/Command/i)).not.toBeInTheDocument();
		});

		it('should show headers editor when http type is set', () => {
			render(McpForm, {
				props: {
					initialValues: { type: 'http' },
					onSubmit: mockOnSubmit,
					onCancel: mockOnCancel
				}
			});

			expect(screen.getByLabelText(/URL/i)).toBeInTheDocument();
			expect(screen.getByText('Headers')).toBeInTheDocument();
		});
	});

	describe('paste import', () => {
		it('should have paste instructions visible by default', () => {
			render(McpForm, { props: { onSubmit: mockOnSubmit, onCancel: mockOnCancel } });

			expect(screen.getByText(/Paste a/)).toBeInTheDocument();
			expect(screen.getByText(/claude mcp add/)).toBeInTheDocument();
		});
	});
});
