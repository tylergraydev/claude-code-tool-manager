import { describe, it, expect } from 'vitest';
import type { McpType, McpSource, ToolContent } from '$lib/types/mcp';

describe('MCP Types', () => {
	describe('McpType', () => {
		it('should define valid McpType values', () => {
			const validTypes: McpType[] = ['stdio', 'sse', 'http'];

			expect(validTypes).toHaveLength(3);
		});

		it('should include stdio type', () => {
			const type: McpType = 'stdio';

			expect(type).toBe('stdio');
		});

		it('should include sse type', () => {
			const type: McpType = 'sse';

			expect(type).toBe('sse');
		});

		it('should include http type', () => {
			const type: McpType = 'http';

			expect(type).toBe('http');
		});
	});

	describe('McpSource', () => {
		it('should define valid McpSource values', () => {
			const validSources: McpSource[] = ['manual', 'auto-detected', 'imported', 'system'];

			expect(validSources).toHaveLength(4);
		});

		it('should include manual source', () => {
			const source: McpSource = 'manual';

			expect(source).toBe('manual');
		});

		it('should include auto-detected source', () => {
			const source: McpSource = 'auto-detected';

			expect(source).toBe('auto-detected');
		});

		it('should include imported source', () => {
			const source: McpSource = 'imported';

			expect(source).toBe('imported');
		});

		it('should include system source', () => {
			const source: McpSource = 'system';

			expect(source).toBe('system');
		});
	});

	describe('ToolContent', () => {
		it('should define text content type', () => {
			const content: ToolContent = { type: 'text', text: 'Hello' };

			expect(content.type).toBe('text');
			expect(content.text).toBe('Hello');
		});

		it('should define image content type', () => {
			const content: ToolContent = {
				type: 'image',
				data: 'base64data',
				mimeType: 'image/png'
			};

			expect(content.type).toBe('image');
			expect(content.data).toBe('base64data');
			expect(content.mimeType).toBe('image/png');
		});

		it('should define resource content type', () => {
			const content: ToolContent = {
				type: 'resource',
				uri: 'file:///path/to/file.txt',
				mimeType: 'text/plain',
				text: 'file content'
			};

			expect(content.type).toBe('resource');
			expect(content.uri).toBe('file:///path/to/file.txt');
			expect(content.mimeType).toBe('text/plain');
			expect(content.text).toBe('file content');
		});

		it('should allow optional mimeType on resource content', () => {
			const content: ToolContent = {
				type: 'resource',
				uri: 'file:///path/to/file.txt'
			};

			expect(content.type).toBe('resource');
			expect(content.uri).toBe('file:///path/to/file.txt');
			expect(content.mimeType).toBeUndefined();
		});
	});
});
