import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	soundLibrary: {
		systemSounds: [],
		customSounds: [],
		isLoading: false,
		isPlaying: null,
		soundsDirectory: '~/.claude/sounds/',
		load: vi.fn(),
		previewSound: vi.fn(),
		uploadSound: vi.fn(),
		deleteSound: vi.fn(),
		getSoundByPath: vi.fn()
	},
	notifications: {
		success: vi.fn(),
		error: vi.fn()
	}
}));

describe('SoundBrowser Component', () => {
	let SoundBrowser: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/sounds/SoundBrowser.svelte');
		SoundBrowser = mod.default;
	});

	it('should render Sound Browser heading', () => {
		render(SoundBrowser);
		expect(screen.getByText('Sound Browser')).toBeInTheDocument();
	});

	it('should show system sounds tab', () => {
		render(SoundBrowser);
		expect(screen.getByText(/System Sounds/)).toBeInTheDocument();
	});

	it('should show custom sounds tab', () => {
		render(SoundBrowser);
		expect(screen.getByText(/Custom Sounds/)).toBeInTheDocument();
	});

	it('should show No system sounds found when empty', () => {
		render(SoundBrowser);
		expect(screen.getByText('No system sounds found')).toBeInTheDocument();
	});

	it('should show close button when onClose provided', () => {
		render(SoundBrowser, { props: { onClose: vi.fn() } });
		// X button should be present
		const buttons = screen.getAllByRole('button');
		expect(buttons.length).toBeGreaterThan(0);
	});

	it('should show sounds directory in footer', () => {
		render(SoundBrowser);
		expect(screen.getByText('~/.claude/sounds/')).toBeInTheDocument();
	});

	it('should render system sounds grid when sounds exist', async () => {
		const { soundLibrary } = await import('$lib/stores');
		(soundLibrary as any).systemSounds = [
			{ name: 'Ping', path: '/System/Sounds/Ping.aiff', category: 'system' },
			{ name: 'Pop', path: '/System/Sounds/Pop.aiff', category: 'system' }
		];
		render(SoundBrowser);
		expect(screen.getByText('Ping')).toBeInTheDocument();
		expect(screen.getByText('Pop')).toBeInTheDocument();
		(soundLibrary as any).systemSounds = [];
	});
});

describe('SoundPicker Component', () => {
	let SoundPicker: any;

	beforeAll(async () => {
		const mod = await import('$lib/components/sounds/SoundPicker.svelte');
		SoundPicker = mod.default;
	});

	it('should render with placeholder', () => {
		render(SoundPicker, {
			props: { value: '', placeholder: 'Pick a sound' }
		});
		expect(screen.getByText('Pick a sound')).toBeInTheDocument();
	});

	it('should render with default placeholder', () => {
		render(SoundPicker, { props: { value: '' } });
		expect(screen.getByText('Select a sound...')).toBeInTheDocument();
	});

	it('should show preview button when value is set', () => {
		render(SoundPicker, {
			props: { value: '/System/Sounds/Ping.aiff' }
		});
		expect(screen.getByTitle('Preview sound')).toBeInTheDocument();
	});

	it('should not show preview button when no value', () => {
		render(SoundPicker, { props: { value: '' } });
		expect(screen.queryByTitle('Preview sound')).not.toBeInTheDocument();
	});
});

describe('Sounds index.ts exports', () => {
	let soundExports: any;

	beforeAll(async () => {
		soundExports = await import('$lib/components/sounds');
	});

	it('should export all components', () => {
		expect(soundExports.SoundBrowser).toBeDefined();
		expect(soundExports.SoundPicker).toBeDefined();
	});
});
