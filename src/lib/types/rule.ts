export interface Rule {
	id: number;
	name: string;
	description?: string;
	content: string;
	paths?: string[];
	tags?: string[];
	source: string;
	sourcePath?: string;
	isSymlink: boolean;
	symlinkTarget?: string;
	isFavorite: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface CreateRuleRequest {
	name: string;
	description?: string;
	content: string;
	paths?: string[];
	tags?: string[];
}

export interface ProjectRule {
	id: number;
	ruleId: number;
	rule: Rule;
	isEnabled: boolean;
}

export interface GlobalRule {
	id: number;
	ruleId: number;
	rule: Rule;
	isEnabled: boolean;
}
