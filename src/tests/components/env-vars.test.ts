import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/types', async (importOriginal) => {
	const actual = await importOriginal() as any;
	return {
		...actual,
		KNOWN_ENV_VARS: actual.KNOWN_ENV_VARS ?? [
			{ key: 'ANTHROPIC_API_KEY', description: 'API Key for Anthropic', category: 'API' }
		],
		ENV_VAR_CATEGORIES: actual.ENV_VAR_CATEGORIES ?? ['API', 'General']
	};
});

describe('EnvVarsEditor Component', () => {
	let EnvVarsEditor: any;

	const mockSettings = {
		scope: 'project',
		availableModels: [],
		env: { MY_VAR: 'my-value' }
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/env-vars/EnvVarsEditor.svelte');
		EnvVarsEditor = mod.default;
	});

	it('should render heading', () => {
		render(EnvVarsEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('Environment Variables')).toBeInTheDocument();
	});

	it('should show existing env vars', () => {
		render(EnvVarsEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		const inputs = document.querySelectorAll('input');
		const keyInput = Array.from(inputs).find(i => (i as HTMLInputElement).value === 'MY_VAR');
		expect(keyInput).toBeTruthy();
	});

	it('should call onsave on save', async () => {
		const onsave = vi.fn();
		render(EnvVarsEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save Environment Variables'));
		expect(onsave).toHaveBeenCalledOnce();
	});
});

describe('KnownEnvVarPicker Component', () => {
	let KnownEnvVarPicker: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/env-vars/KnownEnvVarPicker.svelte');
		KnownEnvVarPicker = mod.default;
	});

	it('should render heading', () => {
		render(KnownEnvVarPicker, {
			props: { existingKeys: [], onselect: vi.fn() }
		});
		expect(screen.getByText('Known Environment Variables')).toBeInTheDocument();
	});
});

describe('Env-vars index.ts exports', () => {
	let envExports: any;

	beforeAll(async () => {
		envExports = await import('$lib/components/env-vars');
	});

	it('should export all components', () => {
		expect(envExports.EnvVarsEditor).toBeDefined();
		expect(envExports.KnownEnvVarPicker).toBeDefined();
	});
});
