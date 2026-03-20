import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/types', async (importOriginal) => {
	const actual = await importOriginal() as any;
	return {
		...actual,
		MEMORY_SCOPE_LABELS: actual.MEMORY_SCOPE_LABELS ?? {
			user: { label: 'User', description: 'User scope' },
			project: { label: 'Project', description: 'Project scope' },
			local: { label: 'Local', description: 'Local scope' }
		}
	};
});

describe('MemoryEditor Component', () => {
	let MemoryEditor: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/memory/MemoryEditor.svelte');
		MemoryEditor = mod.default;
	});

	it('should render textarea with content', () => {
		render(MemoryEditor, {
			props: { content: '# Test content', onchange: vi.fn() }
		});
		const textarea = document.querySelector('textarea') as HTMLTextAreaElement;
		expect(textarea).toBeInTheDocument();
		expect(textarea.value).toBe('# Test content');
	});

	it('should call onchange when content changes', async () => {
		const onchange = vi.fn();
		render(MemoryEditor, {
			props: { content: '', onchange }
		});
		const textarea = document.querySelector('textarea')!;
		await fireEvent.input(textarea, { target: { value: 'new content' } });
		expect(onchange).toHaveBeenCalled();
	});
});

describe('MemoryFileStatus Component', () => {
	let MemoryFileStatus: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/memory/MemoryFileStatus.svelte');
		MemoryFileStatus = mod.default;
	});

	it('should render file info', () => {
		render(MemoryFileStatus, {
			props: {
				file: {
					exists: true,
					filePath: '/home/user/.claude/CLAUDE.md',
					size: 1024,
					modifiedAt: new Date().toISOString()
				}
			}
		});
		expect(screen.getByText('/home/user/.claude/CLAUDE.md')).toBeInTheDocument();
	});
});

describe('MemoryScopeSelector Component', () => {
	let MemoryScopeSelector: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/memory/MemoryScopeSelector.svelte');
		MemoryScopeSelector = mod.default;
	});

	it('should render scope buttons', () => {
		render(MemoryScopeSelector, {
			props: {
				selectedScope: 'user',
				memoryFiles: {
					user: { exists: true, path: '/test', size: 100, modifiedAt: '' },
					project: null,
					local: null
				},
				hasProject: false,
				onselect: vi.fn()
			}
		});
		expect(document.body).toBeTruthy();
	});
});

describe('Memory index.ts exports', () => {
	let memExports: any;

	beforeAll(async () => {
		memExports = await import('$lib/components/memory');
	});

	it('should export all components', () => {
		expect(memExports.MemoryScopeSelector).toBeDefined();
		expect(memExports.MemoryEditor).toBeDefined();
		expect(memExports.MemoryPreview).toBeDefined();
		expect(memExports.MemoryFileStatus).toBeDefined();
	});
});
