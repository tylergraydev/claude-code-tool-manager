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
			network: {},
			filesystem: {}
		}
	};

	it('should render General heading', async () => {
		const { default: SandboxConfigEditor } = await import('$lib/components/sandbox/SandboxConfigEditor.svelte');
		render(SandboxConfigEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('General')).toBeInTheDocument();
	});

	it('should render all section headings', async () => {
		const { default: SandboxConfigEditor } = await import('$lib/components/sandbox/SandboxConfigEditor.svelte');
		render(SandboxConfigEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('General')).toBeInTheDocument();
		expect(screen.getByText('Excluded Commands')).toBeInTheDocument();
		expect(screen.getByText('Network')).toBeInTheDocument();
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

	it('should include sandbox in save payload', async () => {
		const { default: SandboxConfigEditor } = await import('$lib/components/sandbox/SandboxConfigEditor.svelte');
		const onsave = vi.fn();
		render(SandboxConfigEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Sandbox Settings'));
		const saved = onsave.mock.calls[0][0];
		expect(saved.sandbox).toBeDefined();
		expect(saved.sandbox.enabled).toBe(true);
	});

	it('should render tri-state toggle labels', async () => {
		const { default: SandboxConfigEditor } = await import('$lib/components/sandbox/SandboxConfigEditor.svelte');
		render(SandboxConfigEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Sandbox Enabled')).toBeInTheDocument();
		expect(screen.getByText('Auto-Allow Bash if Sandboxed')).toBeInTheDocument();
		expect(screen.getByText('Allow Unsandboxed Commands')).toBeInTheDocument();
		expect(screen.getByText('Enable Weaker Nested Sandbox')).toBeInTheDocument();
	});

	it('should render existing excluded commands', async () => {
		const { default: SandboxConfigEditor } = await import('$lib/components/sandbox/SandboxConfigEditor.svelte');
		render(SandboxConfigEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('rm')).toBeInTheDocument();
	});

	it('should include excluded commands in save', async () => {
		const { default: SandboxConfigEditor } = await import('$lib/components/sandbox/SandboxConfigEditor.svelte');
		const onsave = vi.fn();
		render(SandboxConfigEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Sandbox Settings'));
		expect(onsave.mock.calls[0][0].sandbox.excludedCommands).toEqual(['rm']);
	});

	it('should save undefined sandbox when no values set', async () => {
		const { default: SandboxConfigEditor } = await import('$lib/components/sandbox/SandboxConfigEditor.svelte');
		const onsave = vi.fn();
		const emptySettings = { scope: 'user', availableModels: [], sandbox: undefined };
		render(SandboxConfigEditor, {
			props: { settings: emptySettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Sandbox Settings'));
		expect(onsave.mock.calls[0][0].sandbox).toBeUndefined();
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

	it('should render all network toggles', async () => {
		const { default: SandboxNetworkEditor } = await import('$lib/components/sandbox/SandboxNetworkEditor.svelte');
		render(SandboxNetworkEditor, {
			props: { network: {}, onchange: vi.fn() }
		});
		expect(screen.getByText('Allow All Unix Sockets')).toBeInTheDocument();
		expect(screen.getByText('Allow Local Binding')).toBeInTheDocument();
	});

	it('should render proxy port inputs', async () => {
		const { default: SandboxNetworkEditor } = await import('$lib/components/sandbox/SandboxNetworkEditor.svelte');
		render(SandboxNetworkEditor, {
			props: { network: {}, onchange: vi.fn() }
		});
		expect(screen.getByLabelText('HTTP Proxy Port')).toBeInTheDocument();
		expect(screen.getByLabelText('SOCKS Proxy Port')).toBeInTheDocument();
	});

	it('should display existing port values', async () => {
		const { default: SandboxNetworkEditor } = await import('$lib/components/sandbox/SandboxNetworkEditor.svelte');
		render(SandboxNetworkEditor, {
			props: {
				network: { httpProxyPort: 8080, socksProxyPort: 1080 },
				onchange: vi.fn()
			}
		});
		expect((screen.getByLabelText('HTTP Proxy Port') as HTMLInputElement).value).toBe('8080');
		expect((screen.getByLabelText('SOCKS Proxy Port') as HTMLInputElement).value).toBe('1080');
	});

	it('should call onchange when domain added', async () => {
		const { default: SandboxNetworkEditor } = await import('$lib/components/sandbox/SandboxNetworkEditor.svelte');
		const onchange = vi.fn();
		const { container } = render(SandboxNetworkEditor, {
			props: { network: {}, onchange }
		});
		const domainInput = container.querySelector('input[placeholder="*.example.com"]') as HTMLInputElement;
		await fireEvent.input(domainInput, { target: { value: 'test.com' } });
		await fireEvent.keyDown(domainInput, { key: 'Enter' });
		expect(onchange).toHaveBeenCalled();
		const lastCall = onchange.mock.calls[onchange.mock.calls.length - 1][0];
		expect(lastCall.allowedDomains).toContain('test.com');
	});

	it('should display multiple unix sockets', async () => {
		const { default: SandboxNetworkEditor } = await import('$lib/components/sandbox/SandboxNetworkEditor.svelte');
		render(SandboxNetworkEditor, {
			props: {
				network: { allowUnixSockets: ['/var/run/docker.sock', '/tmp/socket'] },
				onchange: vi.fn()
			}
		});
		expect(screen.getByText('/var/run/docker.sock')).toBeInTheDocument();
		expect(screen.getByText('/tmp/socket')).toBeInTheDocument();
	});
});

describe('SandboxFilesystemEditor Component', () => {
	it('should render existing allowRead paths', async () => {
		const { default: SandboxFilesystemEditor } = await import('$lib/components/sandbox/SandboxFilesystemEditor.svelte');
		render(SandboxFilesystemEditor, {
			props: {
				filesystem: { allowRead: ['/opt/data'] },
				onchange: vi.fn()
			}
		});
		expect(screen.getByText('/opt/data')).toBeInTheDocument();
	});

	it('should render all three sections', async () => {
		const { default: SandboxFilesystemEditor } = await import('$lib/components/sandbox/SandboxFilesystemEditor.svelte');
		render(SandboxFilesystemEditor, {
			props: { filesystem: {}, onchange: vi.fn() }
		});
		expect(screen.getByText('Allow Read')).toBeInTheDocument();
		expect(screen.getByText('Deny Read')).toBeInTheDocument();
		expect(screen.getByText('Allow Unix Sockets')).toBeInTheDocument();
	});

	it('should render existing denyRead paths', async () => {
		const { default: SandboxFilesystemEditor } = await import('$lib/components/sandbox/SandboxFilesystemEditor.svelte');
		render(SandboxFilesystemEditor, {
			props: {
				filesystem: { denyRead: ['/etc/secrets'] },
				onchange: vi.fn()
			}
		});
		expect(screen.getByText('/etc/secrets')).toBeInTheDocument();
	});

	it('should render existing unix sockets', async () => {
		const { default: SandboxFilesystemEditor } = await import('$lib/components/sandbox/SandboxFilesystemEditor.svelte');
		render(SandboxFilesystemEditor, {
			props: {
				filesystem: { allowUnixSockets: ['/var/run/docker.sock'] },
				onchange: vi.fn()
			}
		});
		expect(screen.getByText('/var/run/docker.sock')).toBeInTheDocument();
	});

	it('should call onchange when allowRead path added', async () => {
		const { default: SandboxFilesystemEditor } = await import('$lib/components/sandbox/SandboxFilesystemEditor.svelte');
		const onchange = vi.fn();
		const { container } = render(SandboxFilesystemEditor, {
			props: { filesystem: {}, onchange }
		});
		const allowReadInput = container.querySelector('input[placeholder="/opt/data/**"]') as HTMLInputElement;
		await fireEvent.input(allowReadInput, { target: { value: '/my/path' } });
		await fireEvent.keyDown(allowReadInput, { key: 'Enter' });
		expect(onchange).toHaveBeenCalled();
		const lastCall = onchange.mock.calls[onchange.mock.calls.length - 1][0];
		expect(lastCall.allowRead).toContain('/my/path');
	});

	it('should call onchange when denyRead path added', async () => {
		const { default: SandboxFilesystemEditor } = await import('$lib/components/sandbox/SandboxFilesystemEditor.svelte');
		const onchange = vi.fn();
		const { container } = render(SandboxFilesystemEditor, {
			props: { filesystem: {}, onchange }
		});
		const denyReadInput = container.querySelector('input[placeholder="/etc/secrets/**"]') as HTMLInputElement;
		await fireEvent.input(denyReadInput, { target: { value: '/secret/dir' } });
		await fireEvent.keyDown(denyReadInput, { key: 'Enter' });
		expect(onchange).toHaveBeenCalled();
		const lastCall = onchange.mock.calls[onchange.mock.calls.length - 1][0];
		expect(lastCall.denyRead).toContain('/secret/dir');
	});

	it('should return undefined for empty arrays in onchange', async () => {
		const { default: SandboxFilesystemEditor } = await import('$lib/components/sandbox/SandboxFilesystemEditor.svelte');
		const onchange = vi.fn();
		const { container } = render(SandboxFilesystemEditor, {
			props: { filesystem: { allowRead: ['/path'] }, onchange }
		});
		// Remove the path by clicking the X button
		const removeBtn = container.querySelector('button.text-gray-400');
		if (removeBtn) {
			await fireEvent.click(removeBtn);
			const lastCall = onchange.mock.calls[onchange.mock.calls.length - 1][0];
			expect(lastCall.allowRead).toBeUndefined();
		}
	});
});

describe('Sandbox index.ts exports', () => {
	it('should export all sandbox components', async () => {
		const exports = await import('$lib/components/sandbox');
		expect(exports.SandboxConfigEditor).toBeDefined();
		expect(exports.SandboxNetworkEditor).toBeDefined();
		expect(exports.SandboxFilesystemEditor).toBeDefined();
	});
});
