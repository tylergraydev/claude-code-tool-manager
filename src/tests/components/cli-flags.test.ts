import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

describe('CliStartupFlagsCard Component', () => {
	let CliStartupFlagsCard: any;

	async function getComponent() {
		if (!CliStartupFlagsCard) {
			const mod = await import('$lib/components/cli-flags/CliStartupFlagsCard.svelte');
			CliStartupFlagsCard = mod.default;
		}
		return CliStartupFlagsCard;
	}

	it('should render CLI Startup Flags heading', async () => {
		const C = await getComponent();
		render(C);
		expect(screen.getByText('CLI Startup Flags')).toBeInTheDocument();
	});

	it('should render Scheduling heading', async () => {
		const C = await getComponent();
		render(C);
		expect(screen.getByText('Scheduling')).toBeInTheDocument();
	});

	it('should render all CLI flags', async () => {
		const C = await getComponent();
		render(C);
		expect(screen.getByText('--agent')).toBeInTheDocument();
		expect(screen.getByText('--baremode')).toBeInTheDocument();
		expect(screen.getByText('--system-prompt')).toBeInTheDocument();
		expect(screen.getByText('--append-system-prompt')).toBeInTheDocument();
		expect(screen.getByText('--permissions')).toBeInTheDocument();
		expect(screen.getByText('--allowedTools')).toBeInTheDocument();
		expect(screen.getByText('--disallowedTools')).toBeInTheDocument();
		expect(screen.getByText('--model')).toBeInTheDocument();
		expect(screen.getByText('--max-turns')).toBeInTheDocument();
	});

	it('should render flag checkboxes', async () => {
		const C = await getComponent();
		const { container } = render(C);
		const checkboxes = container.querySelectorAll('input[type="checkbox"]');
		expect(checkboxes.length).toBe(9);
	});

	it('should show command builder when a flag is selected', async () => {
		const C = await getComponent();
		const { container } = render(C);
		const checkboxes = container.querySelectorAll('input[type="checkbox"]');
		// Select --baremode (no arg needed)
		await fireEvent.click(checkboxes[1]);
		expect(screen.getByText('Command')).toBeInTheDocument();
		expect(screen.getByText('Copy')).toBeInTheDocument();
	});

	it('should build correct command for simple flag', async () => {
		const C = await getComponent();
		const { container } = render(C);
		const checkboxes = container.querySelectorAll('input[type="checkbox"]');
		// Select --baremode
		await fireEvent.click(checkboxes[1]);
		expect(screen.getByText('claude --baremode')).toBeInTheDocument();
	});

	it('should show argument input when flag with arg is selected', async () => {
		const C = await getComponent();
		const { container } = render(C);
		const checkboxes = container.querySelectorAll('input[type="checkbox"]');
		// Select --agent (has arg)
		await fireEvent.click(checkboxes[0]);
		const textInputs = container.querySelectorAll('input[type="text"]');
		expect(textInputs.length).toBeGreaterThan(0);
	});

	it('should render scheduling commands', async () => {
		const C = await getComponent();
		render(C);
		expect(screen.getByText('/loop')).toBeInTheDocument();
		expect(screen.getByText('/schedule')).toBeInTheDocument();
	});

	it('should render cron tool references', async () => {
		const C = await getComponent();
		render(C);
		expect(screen.getByText('CronCreate')).toBeInTheDocument();
		expect(screen.getByText('CronList')).toBeInTheDocument();
		expect(screen.getByText('CronDelete')).toBeInTheDocument();
	});

	it('should not show command builder when no flags selected', async () => {
		const C = await getComponent();
		render(C);
		expect(screen.queryByText('Command')).not.toBeInTheDocument();
	});

	it('should deselect flag and hide command builder', async () => {
		const C = await getComponent();
		const { container } = render(C);
		const checkboxes = container.querySelectorAll('input[type="checkbox"]');
		// Select then deselect --baremode
		await fireEvent.click(checkboxes[1]);
		expect(screen.getByText('Command')).toBeInTheDocument();
		await fireEvent.click(checkboxes[1]);
		expect(screen.queryByText('Command')).not.toBeInTheDocument();
	});
});

describe('CLI-flags index.ts exports', () => {
	it('should export CliStartupFlagsCard', async () => {
		const exports = await import('$lib/components/cli-flags');
		expect(exports.CliStartupFlagsCard).toBeDefined();
	});
});
