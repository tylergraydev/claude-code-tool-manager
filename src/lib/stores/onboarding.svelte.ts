const STORAGE_KEY = 'claude-tool-manager-onboarding';
const ALL_STEPS = ['add-project', 'add-mcp', 'assign-mcp', 'explore-settings'] as const;

type OnboardingStep = (typeof ALL_STEPS)[number];

interface OnboardingData {
	dismissed: boolean;
	completedSteps: string[];
}

function loadFromStorage(): { data: OnboardingData | null; isFirstRun: boolean } {
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (raw === null) {
			return { data: null, isFirstRun: true };
		}
		const parsed = JSON.parse(raw) as OnboardingData;
		return { data: parsed, isFirstRun: false };
	} catch {
		return { data: null, isFirstRun: true };
	}
}

function saveToStorage(data: OnboardingData): void {
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
	} catch {
		// Gracefully handle quota exceeded or other errors
	}
}

class OnboardingState {
	#completedSteps = $state<string[]>([]);
	#dismissed = $state(false);
	#isFirstRun = $state(true);

	constructor() {
		const { data, isFirstRun } = loadFromStorage();
		this.#isFirstRun = isFirstRun;
		if (data) {
			this.#completedSteps = data.completedSteps ?? [];
			this.#dismissed = data.dismissed ?? false;
		}
	}

	get isFirstRun(): boolean {
		return this.#isFirstRun;
	}

	get isDismissed(): boolean {
		return this.#dismissed;
	}

	get completedSteps(): string[] {
		return this.#completedSteps;
	}

	get isComplete(): boolean {
		return this.#completedSteps.length >= ALL_STEPS.length;
	}

	get progress(): number {
		return this.#completedSteps.length / ALL_STEPS.length;
	}

	get showOnboarding(): boolean {
		return !this.#dismissed && !this.isComplete;
	}

	completeStep(step: string): void {
		if (this.#completedSteps.includes(step)) return;
		this.#completedSteps = [...this.#completedSteps, step];
		this.#isFirstRun = false;
		this.#save();
	}

	dismiss(): void {
		this.#dismissed = true;
		this.#save();
	}

	syncWithStores(projectCount: number, mcpCount: number, assignmentCount: number): void {
		let changed = false;

		if (projectCount > 0 && !this.#completedSteps.includes('add-project')) {
			this.#completedSteps = [...this.#completedSteps, 'add-project'];
			changed = true;
		}
		if (mcpCount > 0 && !this.#completedSteps.includes('add-mcp')) {
			this.#completedSteps = [...this.#completedSteps, 'add-mcp'];
			changed = true;
		}
		if (assignmentCount > 0 && !this.#completedSteps.includes('assign-mcp')) {
			this.#completedSteps = [...this.#completedSteps, 'assign-mcp'];
			changed = true;
		}

		if (changed) {
			this.#isFirstRun = false;
			this.#save();
		}
	}

	#save(): void {
		saveToStorage({
			dismissed: this.#dismissed,
			completedSteps: this.#completedSteps
		});
	}
}

export const onboarding = new OnboardingState();
