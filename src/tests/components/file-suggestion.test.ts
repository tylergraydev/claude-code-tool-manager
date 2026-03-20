import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

describe('FileSuggestionEditor Component', () => {
	let FileSuggestionEditor: any;

	const mockSettings = {
		scope: 'project',
		availableModels: [],
		fileSuggestionCommand: '/path/to/suggest.sh',
		fileSuggestionType: 'command'
	};

	beforeAll(async () => {
		const mod = await import('$lib/components/file-suggestion/FileSuggestionEditor.svelte');
		FileSuggestionEditor = mod.default;
	});

	it('should render heading and description', () => {
		render(FileSuggestionEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		expect(screen.getByText('File Suggestion')).toBeInTheDocument();
	});

	it('should populate command value', () => {
		render(FileSuggestionEditor, {
			props: { settings: mockSettings as any, onsave: vi.fn() }
		});
		const input = screen.getByLabelText('Command') as HTMLInputElement;
		expect(input.value).toBe('/path/to/suggest.sh');
	});

	it('should call onsave with correct data when save clicked', async () => {
		const onsave = vi.fn();
		render(FileSuggestionEditor, {
			props: { settings: mockSettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save File Suggestion Settings'));
		expect(onsave).toHaveBeenCalledOnce();
		const saved = onsave.mock.calls[0][0];
		expect(saved.fileSuggestionCommand).toBe('/path/to/suggest.sh');
		expect(saved.fileSuggestionType).toBe('command');
	});

	it('should clear fileSuggestion fields when command is empty', async () => {
		const onsave = vi.fn();
		const emptySettings = { ...mockSettings, fileSuggestionCommand: '' };
		render(FileSuggestionEditor, {
			props: { settings: emptySettings as any, onsave }
		});
		await fireEvent.click(screen.getByText('Save File Suggestion Settings'));
		const saved = onsave.mock.calls[0][0];
		expect(saved.fileSuggestionCommand).toBeUndefined();
		expect(saved.fileSuggestionType).toBeUndefined();
	});
});

describe('File-suggestion index.ts exports', () => {
	let fsExports: any;

	beforeAll(async () => {
		fsExports = await import('$lib/components/file-suggestion');
	});

	it('should export FileSuggestionEditor', () => {
		expect(fsExports.FileSuggestionEditor).toBeDefined();
	});
});
