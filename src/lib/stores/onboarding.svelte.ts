export type OnboardingStep = 'add-project' | 'add-mcp' | 'assign-mcp' | 'explore-settings';

const TOTAL_STEPS = 4;

class OnboardingState {
	completedSteps = $state<OnboardingStep[]>([]);
	dismissed = $state(false);
	isFirstRun = $state(true);

	progress = $derived(this.completedSteps.length / TOTAL_STEPS);

	showOnboarding = $derived(!this.dismissed && this.completedSteps.length < TOTAL_STEPS);

	completeStep(step: OnboardingStep) {
		if (!this.completedSteps.includes(step)) {
			this.completedSteps = [...this.completedSteps, step];
		}
	}

	dismiss() {
		this.dismissed = true;
	}

	syncWithStores(projectCount: number, mcpCount: number, globalMcpCount: number) {
		if (projectCount > 0) this.completeStep('add-project');
		if (mcpCount > 0) this.completeStep('add-mcp');
		if (globalMcpCount > 0) this.completeStep('assign-mcp');
		this.isFirstRun = projectCount === 0 && mcpCount === 0;
	}
}

export const onboarding = new OnboardingState();
