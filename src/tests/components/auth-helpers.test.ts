import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

describe('AuthHelpersEditor Component', () => {
	const mockSettings = {
		scope: 'project',
		availableModels: [],
		apiKeyHelper: '/path/to/key.sh',
		otelHeadersHelper: '',
		awsAuthRefresh: '',
		awsCredentialExport: ''
	};

	it('should render all auth helper fields', async () => {
		const { default: AuthHelpersEditor } = await import('$lib/components/auth-helpers/AuthHelpersEditor.svelte');
		render(AuthHelpersEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Auth & API Key Helpers')).toBeInTheDocument();
		expect(screen.getByLabelText('API Key Helper')).toBeInTheDocument();
		expect(screen.getByLabelText('OpenTelemetry Headers Helper')).toBeInTheDocument();
		expect(screen.getByLabelText('AWS Auth Refresh')).toBeInTheDocument();
		expect(screen.getByLabelText('AWS Credential Export')).toBeInTheDocument();
	});

	it('should populate initial values', async () => {
		const { default: AuthHelpersEditor } = await import('$lib/components/auth-helpers/AuthHelpersEditor.svelte');
		render(AuthHelpersEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		const input = screen.getByLabelText('API Key Helper') as HTMLInputElement;
		expect(input.value).toBe('/path/to/key.sh');
	});

	it('should call onsave when save button clicked', async () => {
		const { default: AuthHelpersEditor } = await import('$lib/components/auth-helpers/AuthHelpersEditor.svelte');
		const onsave = vi.fn();
		render(AuthHelpersEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Auth Helper Settings'));
		expect(onsave).toHaveBeenCalledOnce();
	});
});

describe('Auth-helpers index.ts exports', () => {
	it('should export AuthHelpersEditor', async () => {
		const exports = await import('$lib/components/auth-helpers');
		expect(exports.AuthHelpersEditor).toBeDefined();
	});
});
