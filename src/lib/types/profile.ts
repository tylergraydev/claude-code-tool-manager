export interface Profile {
	id: number;
	name: string;
	description: string | null;
	icon: string | null;
	isActive: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface CreateProfileRequest {
	name: string;
	description?: string | null;
	icon?: string | null;
}

export interface ProfileItem {
	id: number;
	profileId: number;
	itemType: string;
	itemId: number;
	createdAt: string;
}

export interface ProfileWithItems {
	profile: Profile;
	mcps: number[];
	skills: number[];
	commands: number[];
	subagents: number[];
	hooks: number[];
}
