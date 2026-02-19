export interface InsightsReportInfo {
	filePath: string;
	exists: boolean;
	htmlContent: string | null;
}

export interface SessionFacetsInfo {
	dirPath: string;
	exists: boolean;
	facets: SessionFacet[];
}

export interface SessionFacet {
	sessionId: string;
	underlyingGoal: string;
	goalCategories: Record<string, number>;
	outcome: string;
	userSatisfactionCounts: Record<string, number>;
	claudeHelpfulness: string;
	sessionType: string;
	frictionCounts: Record<string, number>;
	frictionDetail: string;
	primarySuccess: string;
	briefSummary: string;
}

export type SessionOutcome =
	| 'fully_achieved'
	| 'mostly_achieved'
	| 'partially_achieved'
	| 'not_achieved'
	| 'abandoned'
	| 'unknown';

export type SatisfactionLevel =
	| 'satisfied'
	| 'likely_satisfied'
	| 'neutral'
	| 'likely_unsatisfied'
	| 'unsatisfied';

export type HelpfulnessLevel =
	| 'essential'
	| 'very_helpful'
	| 'helpful'
	| 'slightly_helpful'
	| 'not_helpful';

export type SessionType =
	| 'single_task'
	| 'multi_task'
	| 'exploration'
	| 'debugging'
	| 'learning'
	| 'unknown';

export const OUTCOME_LABELS: Record<string, string> = {
	fully_achieved: 'Fully Achieved',
	mostly_achieved: 'Mostly Achieved',
	partially_achieved: 'Partially Achieved',
	not_achieved: 'Not Achieved',
	abandoned: 'Abandoned',
	unknown: 'Unknown'
};

export const OUTCOME_COLORS: Record<string, string> = {
	fully_achieved: '#10b981',
	mostly_achieved: '#3b82f6',
	partially_achieved: '#f59e0b',
	not_achieved: '#ef4444',
	abandoned: '#6b7280',
	unknown: '#9ca3af'
};

export const HELPFULNESS_LABELS: Record<string, string> = {
	essential: 'Essential',
	very_helpful: 'Very Helpful',
	helpful: 'Helpful',
	slightly_helpful: 'Slightly Helpful',
	not_helpful: 'Not Helpful'
};

export const HELPFULNESS_COLORS: Record<string, string> = {
	essential: '#8b5cf6',
	very_helpful: '#3b82f6',
	helpful: '#10b981',
	slightly_helpful: '#f59e0b',
	not_helpful: '#ef4444'
};

export const SESSION_TYPE_LABELS: Record<string, string> = {
	single_task: 'Single Task',
	multi_task: 'Multi Task',
	exploration: 'Exploration',
	debugging: 'Debugging',
	learning: 'Learning',
	unknown: 'Unknown'
};

export const FRICTION_LABELS: Record<string, string> = {
	tool_failures: 'Tool Failures',
	context_loss: 'Context Loss',
	misunderstanding: 'Misunderstanding',
	slow_response: 'Slow Response',
	permission_issues: 'Permission Issues',
	unclear_instructions: 'Unclear Instructions',
	repeated_attempts: 'Repeated Attempts',
	scope_creep: 'Scope Creep'
};
