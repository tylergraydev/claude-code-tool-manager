export interface Skill {
	id: number;
	name: string;
	description?: string;
	content: string;
	allowedTools?: string[];
	model?: string;
	disableModelInvocation: boolean;
	tags?: string[];
	source: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateSkillRequest {
	name: string;
	description?: string;
	content: string;
	allowedTools?: string[];
	model?: string;
	disableModelInvocation?: boolean;
	tags?: string[];
}

export interface ProjectSkill {
	id: number;
	skillId: number;
	skill: Skill;
	isEnabled: boolean;
}

export interface GlobalSkill {
	id: number;
	skillId: number;
	skill: Skill;
	isEnabled: boolean;
}

// Skill Files (references, assets, scripts)
export type SkillFileType = 'reference' | 'asset' | 'script';

export interface SkillFile {
	id: number;
	skillId: number;
	fileType: SkillFileType;
	name: string;
	content: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateSkillFileRequest {
	skillId: number;
	fileType: SkillFileType;
	name: string;
	content: string;
}
