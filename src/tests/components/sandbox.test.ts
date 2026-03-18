import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

describe('SandboxConfigEditor Component', () => {
	const mockSettings = {
		scope: 'project',
		availableModels: [],
		sandbox: {
			enabled: true,
			autoAllowBashIfSandboxed: false,
			excludedCommands: ['rm'],
			network: {}
		}
	};

	it('should render Security heading', async () => {
		const { default: SandboxConfigEditor } = await import('$lib/components/sandbox/SandboxConfigEditor.svelte');
		render(SandboxConfigEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('General')).toBeInTheDocument();
	});

	it('should render save button', async () => {
		const { default: SandboxConfigEditor } = await import('$lib/components/sandbox/SandboxConfigEditor.svelte');
		render(SandboxConfigEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Save Sandbox Settings')).toBeInTheDocument();
	});

	it('should call onsave on save', async () => {
		const { default: SandboxConfigEditor } = await import('$lib/components/sandbox/SandboxConfigEditor.svelte');
		const onsave = vi.fn();
		render(SandboxConfigEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Sandbox Settings'));
		expect(onsave).toHaveBeenCalledOnce();
	});
});

describe('SandboxNetworkEditor Component', () => {
	it('should render network settings', async () => {
		const { default: SandboxNetworkEditor } = await import('$lib/components/sandbox/SandboxNetworkEditor.svelte');
		render(SandboxNetworkEditor, {
			props: {
				network: { allowedDomains: ['example.com'] },
				onchange: vi.fn()
			}
		});
		expect(screen.getByText('example.com')).toBeInTheDocument();
	});
});

describe('Sandbox index.ts exports', () => {
	it('should export all sandbox components', async () => {
		const exports = await import('$lib/components/sandbox');
		expect(exports.SandboxConfigEditor).toBeDefined();
		expect(exports.SandboxNetworkEditor).toBeDefined();
	});
});
