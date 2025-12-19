export type SkillType = 'command' | 'skill';

export interface Skill {
	id: number;
	name: string;
	description?: string;
	content: string;
	skillType: SkillType;
	allowedTools?: string[];
	argumentHint?: string;
	tags?: string[];
	source: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateSkillRequest {
	name: string;
	description?: string;
	content: string;
	skillType: SkillType;
	allowedTools?: string[];
	argumentHint?: string;
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
