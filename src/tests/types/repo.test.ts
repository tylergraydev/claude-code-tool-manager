import { describe, it, expect } from 'vitest';
import type { RepoType, ContentType, ItemType } from '$lib/types/repo';

describe('Repo Types', () => {
	describe('RepoType', () => {
		it('should define valid RepoType values', () => {
			const validTypes: RepoType[] = ['file_based', 'readme_based'];

			expect(validTypes).toHaveLength(2);
		});

		it('should include file_based type', () => {
			const type: RepoType = 'file_based';

			expect(type).toBe('file_based');
		});

		it('should include readme_based type', () => {
			const type: RepoType = 'readme_based';

			expect(type).toBe('readme_based');
		});
	});

	describe('ContentType', () => {
		it('should define valid ContentType values', () => {
			const validTypes: ContentType[] = ['mcp', 'skill', 'subagent', 'mixed'];

			expect(validTypes).toHaveLength(4);
		});

		it('should include mcp type', () => {
			const type: ContentType = 'mcp';

			expect(type).toBe('mcp');
		});

		it('should include skill type', () => {
			const type: ContentType = 'skill';

			expect(type).toBe('skill');
		});

		it('should include subagent type', () => {
			const type: ContentType = 'subagent';

			expect(type).toBe('subagent');
		});

		it('should include mixed type', () => {
			const type: ContentType = 'mixed';

			expect(type).toBe('mixed');
		});
	});

	describe('ItemType', () => {
		it('should define valid ItemType values', () => {
			const validTypes: ItemType[] = ['mcp', 'skill', 'subagent'];

			expect(validTypes).toHaveLength(3);
		});

		it('should include mcp type', () => {
			const type: ItemType = 'mcp';

			expect(type).toBe('mcp');
		});

		it('should include skill type', () => {
			const type: ItemType = 'skill';

			expect(type).toBe('skill');
		});

		it('should include subagent type', () => {
			const type: ItemType = 'subagent';

			expect(type).toBe('subagent');
		});
	});
});
