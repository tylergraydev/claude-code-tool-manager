import { describe, it, expect } from 'vitest';
import { parseSkillMarkdown, parseSubAgentMarkdown } from '$lib/utils/markdownParser';

describe('Markdown Parser', () => {
	describe('parseSkillMarkdown', () => {
		describe('valid skill parsing', () => {
			it('should parse skill with all fields', () => {
				const markdown = `---
name: my-skill
description: A helpful skill
allowed-tools: Read, Write, Edit
argument-hint: [file] [--verbose]
skill-type: command
tags: utility, file-ops
---
This is the skill content.

It can have multiple lines.`;

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data).toBeDefined();
				expect(result.data?.name).toBe('my-skill');
				expect(result.data?.description).toBe('A helpful skill');
				expect(result.data?.content).toBe('This is the skill content.\n\nIt can have multiple lines.');
				expect(result.data?.skillType).toBe('command');
				expect(result.data?.allowedTools).toEqual(['Read', 'Write', 'Edit']);
				expect(result.data?.argumentHint).toBe('[file] [--verbose]');
				expect(result.data?.tags).toEqual(['utility', 'file-ops']);
			});

			it('should parse skill with minimal fields', () => {
				const markdown = `---
name: simple-skill
---
Just the content.`;

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.name).toBe('simple-skill');
				expect(result.data?.content).toBe('Just the content.');
				expect(result.data?.description).toBeUndefined();
				expect(result.data?.skillType).toBe('command'); // default
			});

			it('should parse agent skill type', () => {
				const markdown = `---
name: code-reviewer
skill-type: skill
description: Reviews code for issues
---
Review the code and provide feedback.`;

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.skillType).toBe('skill');
			});

			it('should handle skillType camelCase format', () => {
				const markdown = `---
name: test-skill
skillType: skill
---
Content here.`;

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.skillType).toBe('skill');
			});

			it('should handle allowedTools camelCase format', () => {
				const markdown = `---
name: test-skill
allowedTools: Bash, Read
---
Content here.`;

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.allowedTools).toEqual(['Bash', 'Read']);
			});

			it('should handle argumentHint camelCase format', () => {
				const markdown = `---
name: test-skill
argumentHint: <filename>
---
Content here.`;

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.argumentHint).toBe('<filename>');
			});

			it('should parse space-separated allowed-tools', () => {
				const markdown = `---
name: test-skill
allowed-tools: Read Write Edit
---
Content.`;

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.allowedTools).toEqual(['Read', 'Write', 'Edit']);
			});

			it('should parse mixed comma and space separated tools', () => {
				const markdown = `---
name: test-skill
allowed-tools: Read, Write Edit, Bash
---
Content.`;

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.allowedTools).toEqual(['Read', 'Write', 'Edit', 'Bash']);
			});
		});

		describe('fallback behavior', () => {
			it('should fallback to plain content without frontmatter', () => {
				const markdown = 'Just some plain content without any frontmatter.';

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.name).toBe('');
				expect(result.data?.content).toBe('Just some plain content without any frontmatter.');
			});

			it('should handle content with multiple paragraphs', () => {
				const markdown = `First paragraph.

Second paragraph.

Third paragraph.`;

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.content).toBe(markdown);
			});
		});

		describe('error handling', () => {
			it('should fail for missing name in frontmatter', () => {
				const markdown = `---
description: No name field
---
Content here.`;

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(false);
				expect(result.error).toContain('Missing required field: name');
			});

			it('should fail for missing content after frontmatter', () => {
				const markdown = `---
name: test-skill
---`;

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(false);
				expect(result.error).toContain('Missing content after frontmatter');
			});

			it('should fail for empty content', () => {
				const markdown = '';

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(false);
				expect(result.error).toContain('Could not parse skill markdown');
			});

			it('should fail for whitespace-only content', () => {
				const markdown = '   \n\n   ';

				const result = parseSkillMarkdown(markdown);

				expect(result.success).toBe(false);
				expect(result.error).toContain('Could not parse skill markdown');
			});

			it('should fail for unclosed frontmatter', () => {
				const markdown = `---
name: test
This is content but frontmatter was never closed`;

				const result = parseSkillMarkdown(markdown);

				// Falls back to plain content since frontmatter is invalid
				expect(result.success).toBe(true);
				expect(result.data?.name).toBe('');
				expect(result.data?.content).toContain('name: test');
			});
		});
	});

	describe('parseSubAgentMarkdown', () => {
		describe('valid sub-agent parsing', () => {
			it('should parse sub-agent with all fields', () => {
				const markdown = `---
name: code-reviewer
description: Expert code review agent
tools: Read, Write, Edit, Grep
model: haiku
tags: code, review, quality
---
You are an expert code reviewer. Analyze the code for:
- Bugs and errors
- Performance issues
- Best practices`;

				const result = parseSubAgentMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data).toBeDefined();
				expect(result.data?.name).toBe('code-reviewer');
				expect(result.data?.description).toBe('Expert code review agent');
				expect(result.data?.tools).toEqual(['Read', 'Write', 'Edit', 'Grep']);
				expect(result.data?.model).toBe('haiku');
				expect(result.data?.tags).toEqual(['code', 'review', 'quality']);
				expect(result.data?.content).toContain('You are an expert code reviewer');
			});

			it('should parse sub-agent with minimal fields', () => {
				const markdown = `---
name: simple-agent
description: A simple agent
---
Just do the task.`;

				const result = parseSubAgentMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.name).toBe('simple-agent');
				expect(result.data?.description).toBe('A simple agent');
				expect(result.data?.content).toBe('Just do the task.');
				expect(result.data?.tools).toBeUndefined();
				expect(result.data?.model).toBeUndefined();
			});

			it('should parse sub-agent with model only', () => {
				const markdown = `---
name: fast-agent
description: Uses fast model
model: haiku
---
Quick responses.`;

				const result = parseSubAgentMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.model).toBe('haiku');
				expect(result.data?.tools).toBeUndefined();
			});

			it('should parse sub-agent with tools only', () => {
				const markdown = `---
name: file-agent
description: Works with files
tools: Read, Write
---
Handle file operations.`;

				const result = parseSubAgentMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.tools).toEqual(['Read', 'Write']);
				expect(result.data?.model).toBeUndefined();
			});

			it('should handle multiline content', () => {
				const markdown = `---
name: complex-agent
description: Does complex things
---
First line.

Second line with details.

Third line with more.`;

				const result = parseSubAgentMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.content).toBe('First line.\n\nSecond line with details.\n\nThird line with more.');
			});
		});

		describe('fallback behavior', () => {
			it('should fallback to plain content without frontmatter', () => {
				const markdown = 'Plain agent content without frontmatter.';

				const result = parseSubAgentMarkdown(markdown);

				expect(result.success).toBe(true);
				expect(result.data?.name).toBe('');
				expect(result.data?.description).toBe('');
				expect(result.data?.content).toBe('Plain agent content without frontmatter.');
			});
		});

		describe('error handling', () => {
			it('should fail for missing name', () => {
				const markdown = `---
description: Has description but no name
---
Content here.`;

				const result = parseSubAgentMarkdown(markdown);

				expect(result.success).toBe(false);
				expect(result.error).toContain('Missing required field: name');
			});

			it('should fail for missing description', () => {
				const markdown = `---
name: agent-without-desc
---
Content here.`;

				const result = parseSubAgentMarkdown(markdown);

				expect(result.success).toBe(false);
				expect(result.error).toContain('Missing required field: description');
			});

			it('should fail for missing content after frontmatter', () => {
				const markdown = `---
name: empty-agent
description: Has no content
---`;

				const result = parseSubAgentMarkdown(markdown);

				expect(result.success).toBe(false);
				expect(result.error).toContain('Missing content after frontmatter');
			});

			it('should fail for empty input', () => {
				const result = parseSubAgentMarkdown('');

				expect(result.success).toBe(false);
				expect(result.error).toContain('Could not parse sub-agent markdown');
			});

			it('should fail for whitespace-only input', () => {
				const result = parseSubAgentMarkdown('   \n\t  ');

				expect(result.success).toBe(false);
				expect(result.error).toContain('Could not parse sub-agent markdown');
			});
		});
	});

	describe('edge cases', () => {
		it('should handle Windows line endings (CRLF)', () => {
			const markdown = '---\r\nname: test-skill\r\n---\r\nContent with CRLF.';

			const result = parseSkillMarkdown(markdown);

			expect(result.success).toBe(true);
			expect(result.data?.name).toBe('test-skill');
		});

		it('should handle extra whitespace in frontmatter', () => {
			const markdown = `---
name:   spaced-skill
description:   Has extra spaces
---
Content.`;

			const result = parseSkillMarkdown(markdown);

			expect(result.success).toBe(true);
			expect(result.data?.name).toBe('spaced-skill');
			expect(result.data?.description).toBe('Has extra spaces');
		});

		it('should handle colons in values', () => {
			const markdown = `---
name: test-skill
description: URL: https://example.com
---
Content.`;

			const result = parseSkillMarkdown(markdown);

			expect(result.success).toBe(true);
			expect(result.data?.description).toBe('URL: https://example.com');
		});

		it('should handle empty tags list', () => {
			const markdown = `---
name: test-skill
tags:
---
Content.`;

			const result = parseSkillMarkdown(markdown);

			expect(result.success).toBe(true);
			// Empty value should result in undefined or empty array
			expect(result.data?.tags).toBeUndefined();
		});

		it('should handle single tag', () => {
			const markdown = `---
name: test-skill
tags: single-tag
---
Content.`;

			const result = parseSkillMarkdown(markdown);

			expect(result.success).toBe(true);
			expect(result.data?.tags).toEqual(['single-tag']);
		});

		it('should preserve content formatting', () => {
			const markdown = `---
name: format-test
---
# Header

- List item 1
- List item 2

\`\`\`javascript
const x = 1;
\`\`\``;

			const result = parseSkillMarkdown(markdown);

			expect(result.success).toBe(true);
			expect(result.data?.content).toContain('# Header');
			expect(result.data?.content).toContain('- List item 1');
			expect(result.data?.content).toContain('```javascript');
		});
	});
});
