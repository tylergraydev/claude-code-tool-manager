import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import McpTypeSelector from '$lib/components/mcp/McpTypeSelector.svelte';

describe('McpTypeSelector', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('rendering', () => {
		it('should render connection type label', () => {
			render(McpTypeSelector, { props: { value: 'stdio' } });

			expect(screen.getByText('Connection Type')).toBeInTheDocument();
		});

		it('should render all three type options', () => {
			render(McpTypeSelector, { props: { value: 'stdio' } });

			expect(screen.getByText('Standard I/O')).toBeInTheDocument();
			expect(screen.getByText('Server-Sent Events')).toBeInTheDocument();
			expect(screen.getByText('HTTP/REST')).toBeInTheDocument();
		});

		it('should render type descriptions', () => {
			render(McpTypeSelector, { props: { value: 'stdio' } });

			expect(screen.getByText('Local command-line tool (npx, python, etc.)')).toBeInTheDocument();
			expect(screen.getByText('Cloud service with SSE endpoint')).toBeInTheDocument();
			expect(screen.getByText('REST API with token authentication')).toBeInTheDocument();
		});

		it('should render required indicator', () => {
			render(McpTypeSelector, { props: { value: 'stdio' } });

			expect(screen.getByText('*')).toBeInTheDocument();
		});
	});

	describe('selection', () => {
		it('should highlight stdio when selected', () => {
			const { container } = render(McpTypeSelector, { props: { value: 'stdio' } });

			const stdioButton = screen
				.getByText('Standard I/O')
				.closest('button') as HTMLButtonElement;
			expect(stdioButton.className).toContain('border-primary-500');
		});

		it('should highlight sse when selected', () => {
			const { container } = render(McpTypeSelector, { props: { value: 'sse' } });

			const sseButton = screen
				.getByText('Server-Sent Events')
				.closest('button') as HTMLButtonElement;
			expect(sseButton.className).toContain('border-primary-500');
		});

		it('should highlight http when selected', () => {
			const { container } = render(McpTypeSelector, { props: { value: 'http' } });

			const httpButton = screen
				.getByText('HTTP/REST')
				.closest('button') as HTMLButtonElement;
			expect(httpButton.className).toContain('border-primary-500');
		});
	});

	describe('type switching', () => {
		it('should have clickable stdio button', () => {
			render(McpTypeSelector, { props: { value: 'sse' } });

			const stdioButton = screen.getByText('Standard I/O').closest('button');
			expect(stdioButton).toBeInTheDocument();
			expect(stdioButton).not.toBeDisabled();
		});

		it('should have clickable sse button', () => {
			render(McpTypeSelector, { props: { value: 'stdio' } });

			const sseButton = screen.getByText('Server-Sent Events').closest('button');
			expect(sseButton).toBeInTheDocument();
			expect(sseButton).not.toBeDisabled();
		});

		it('should have clickable http button', () => {
			render(McpTypeSelector, { props: { value: 'stdio' } });

			const httpButton = screen.getByText('HTTP/REST').closest('button');
			expect(httpButton).toBeInTheDocument();
			expect(httpButton).not.toBeDisabled();
		});
	});

	describe('grid layout', () => {
		it('should render buttons in a 3-column grid', () => {
			const { container } = render(McpTypeSelector, { props: { value: 'stdio' } });

			const grid = container.querySelector('.grid-cols-3');
			expect(grid).toBeInTheDocument();
		});
	});
});
