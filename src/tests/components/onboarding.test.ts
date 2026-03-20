import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('$lib/stores/onboarding.svelte', () => ({
	onboarding: {
		isComplete: false,
		isDismissed: false,
		completedSteps: [],
		syncWithStores: vi.fn(),
		dismiss: vi.fn(),
		isStepComplete: vi.fn().mockReturnValue(false)
	}
}));

describe('WelcomeHero Component', () => {
	it('should render welcome message', async () => {
		const { default: WelcomeHero } = await import('$lib/components/onboarding/WelcomeHero.svelte');
		render(WelcomeHero, {
			props: { projectCount: 0, mcpCount: 0, globalMcpCount: 0 }
		});
		expect(screen.getByText(/Getting Started/i)).toBeInTheDocument();
	});

	it('should show steps', async () => {
		const { default: WelcomeHero } = await import('$lib/components/onboarding/WelcomeHero.svelte');
		render(WelcomeHero, {
			props: { projectCount: 0, mcpCount: 0, globalMcpCount: 0 }
		});
		expect(screen.getByText('Add a project')).toBeInTheDocument();
		expect(screen.getByText('Add an MCP server')).toBeInTheDocument();
	});
});
