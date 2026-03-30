class OnboardingState {
	completed = $state(false);
	currentStep = $state(0);
}

export const onboarding = new OnboardingState();
