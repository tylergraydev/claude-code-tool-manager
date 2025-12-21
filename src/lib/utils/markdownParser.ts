/**
 * Parser for Skill and Sub-Agent markdown files with YAML frontmatter
 *
 * Skill Format (Command - .claude/commands/name.md):
 * ---
 * description: What it does
 * allowed-tools: Read, Write, Edit
 * argument-hint: [file] [--verbose]
 * ---
 * Content here...
 *
 * Skill Format (Agent Skill - .claude/skills/name/SKILL.md):
 * ---
 * name: skill-name
 * description: What it does
 * allowed-tools: Read, Grep, Glob
 * ---
 * Content here...
 *
 * Sub-Agent Format (.claude/agents/name.md):
 * ---
 * name: my-agent
 * description: When to invoke
 * tools: Read, Write, Edit
 * model: haiku
 * ---
 * Content here...
 */

import type { SkillType } from '$lib/types';

export interface ParsedSkill {
	name: string;
	description?: string;
	content: string;
	skillType?: SkillType;
	allowedTools?: string[];
	argumentHint?: string;
	model?: string;
	disableModelInvocation?: boolean;
	tags?: string[];
}

export interface ParsedSubAgent {
	name: string;
	description: string;
	content: string;
	tools?: string[];
	model?: string;
	permissionMode?: string;
	skills?: string[];
	tags?: string[];
}

export interface ParseResult<T> {
	success: boolean;
	data?: T;
	error?: string;
}

/**
 * Parse YAML frontmatter from markdown content
 */
function parseFrontmatter(text: string): { frontmatter: Record<string, string>; content: string } | null {
	const trimmed = text.trim();

	// Check for frontmatter delimiter
	if (!trimmed.startsWith('---')) {
		return null;
	}

	// Find the closing delimiter
	const endIndex = trimmed.indexOf('---', 3);
	if (endIndex === -1) {
		return null;
	}

	const frontmatterBlock = trimmed.slice(3, endIndex).trim();
	const content = trimmed.slice(endIndex + 3).trim();

	// Parse YAML-like key: value pairs
	const frontmatter: Record<string, string> = {};
	const lines = frontmatterBlock.split('\n');

	for (const line of lines) {
		const colonIndex = line.indexOf(':');
		if (colonIndex === -1) continue;

		const key = line.slice(0, colonIndex).trim();
		const value = line.slice(colonIndex + 1).trim();

		if (key && value) {
			frontmatter[key] = value;
		}
	}

	return { frontmatter, content };
}

/**
 * Parse a skill from markdown content
 */
export function parseSkillMarkdown(text: string): ParseResult<ParsedSkill> {
	const trimmed = text.trim();

	// Try parsing as frontmatter format
	const parsed = parseFrontmatter(trimmed);

	if (parsed) {
		const { frontmatter, content } = parsed;

		if (!frontmatter.name) {
			return { success: false, error: 'Missing required field: name' };
		}

		if (!content) {
			return { success: false, error: 'Missing content after frontmatter' };
		}

		// Parse allowed-tools (can be comma-separated or space-separated)
		const allowedToolsRaw = frontmatter['allowed-tools'] || frontmatter['allowedTools'];
		const allowedTools = allowedToolsRaw
			?.split(/[,\s]+/)
			.map(t => t.trim())
			.filter(t => t.length > 0);

		// Determine skill type from frontmatter or infer from structure
		let skillType: SkillType = 'command';
		if (frontmatter['skill-type'] || frontmatter['skillType']) {
			const typeValue = (frontmatter['skill-type'] || frontmatter['skillType']).toLowerCase();
			skillType = typeValue === 'skill' ? 'skill' : 'command';
		}

		// Parse disable-model-invocation (can be true/false string)
		const disableModelInvocationRaw = frontmatter['disable-model-invocation'] || frontmatter['disableModelInvocation'];
		const disableModelInvocation = disableModelInvocationRaw?.toLowerCase() === 'true';

		const skill: ParsedSkill = {
			name: frontmatter.name,
			description: frontmatter.description,
			content,
			skillType,
			allowedTools,
			argumentHint: frontmatter['argument-hint'] || frontmatter['argumentHint'],
			model: frontmatter.model,
			disableModelInvocation,
			tags: frontmatter.tags?.split(',').map(t => t.trim()).filter(t => t.length > 0)
		};

		return { success: true, data: skill };
	}

	// Fallback: treat entire content as the skill content, require manual name entry
	if (trimmed.length > 0) {
		return {
			success: true,
			data: {
				name: '',
				content: trimmed
			}
		};
	}

	return { success: false, error: 'Could not parse skill markdown' };
}

/**
 * Parse a sub-agent from markdown content
 */
export function parseSubAgentMarkdown(text: string): ParseResult<ParsedSubAgent> {
	const trimmed = text.trim();

	// Try parsing as frontmatter format
	const parsed = parseFrontmatter(trimmed);

	if (parsed) {
		const { frontmatter, content } = parsed;

		if (!frontmatter.name) {
			return { success: false, error: 'Missing required field: name' };
		}

		if (!frontmatter.description) {
			return { success: false, error: 'Missing required field: description' };
		}

		if (!content) {
			return { success: false, error: 'Missing content after frontmatter' };
		}

		const subagent: ParsedSubAgent = {
			name: frontmatter.name,
			description: frontmatter.description,
			content,
			tools: frontmatter.tools?.split(',').map(t => t.trim()).filter(t => t.length > 0),
			model: frontmatter.model,
			permissionMode: frontmatter.permissionMode || frontmatter['permission-mode'],
			skills: frontmatter.skills?.split(',').map(t => t.trim()).filter(t => t.length > 0),
			tags: frontmatter.tags?.split(',').map(t => t.trim()).filter(t => t.length > 0)
		};

		return { success: true, data: subagent };
	}

	// Fallback: treat entire content as the sub-agent content, require manual name/description entry
	if (trimmed.length > 0) {
		return {
			success: true,
			data: {
				name: '',
				description: '',
				content: trimmed
			}
		};
	}

	return { success: false, error: 'Could not parse sub-agent markdown' };
}

// Legacy alias for backward compatibility
export const parseAgentMarkdown = parseSubAgentMarkdown;
export type ParsedAgent = ParsedSubAgent;
