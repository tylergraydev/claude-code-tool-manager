import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';

vi.mock('$lib/stores', () => ({
	profileLibrary: {
		profiles: [],
		filteredProfiles: [],
		isLoading: false,
		searchQuery: '',
		setSearch: vi.fn()
	}
}));

describe('ProfileCard Component', () => {
	const mockProfile = {
		id: 1,
		name: 'Work',
		description: 'Work profile',
		icon: null,
		isActive: false,
		createdAt: '2024-01-01',
		updatedAt: '2024-01-01'
	};

	it('should render profile name', async () => {
		const { default: ProfileCard } = await import('$lib/components/profiles/ProfileCard.svelte');
		render(ProfileCard, { props: { profile: mockProfile } });
		expect(screen.getByText('Work')).toBeInTheDocument();
	});

	it('should render description', async () => {
		const { default: ProfileCard } = await import('$lib/components/profiles/ProfileCard.svelte');
		render(ProfileCard, { props: { profile: mockProfile } });
		expect(screen.getByText('Work profile')).toBeInTheDocument();
	});

	it('should show active state styling', async () => {
		const { default: ProfileCard } = await import('$lib/components/profiles/ProfileCard.svelte');
		const { container } = render(ProfileCard, {
			props: { profile: { ...mockProfile, isActive: true } }
		});
		const card = container.firstElementChild;
		expect(card?.className).toContain('border-green-300');
	});

	it('should show custom icon when provided', async () => {
		const { default: ProfileCard } = await import('$lib/components/profiles/ProfileCard.svelte');
		render(ProfileCard, {
			props: { profile: { ...mockProfile, icon: '🎨' } }
		});
		expect(screen.getByText('🎨')).toBeInTheDocument();
	});
});

describe('ProfileForm Component', () => {
	it('should render name input', async () => {
		const { default: ProfileForm } = await import('$lib/components/profiles/ProfileForm.svelte');
		render(ProfileForm, {
			props: { onSubmit: vi.fn(), onCancel: vi.fn() }
		});
		expect(screen.getByLabelText(/Name/)).toBeInTheDocument();
	});

	it('should call onCancel', async () => {
		const { default: ProfileForm } = await import('$lib/components/profiles/ProfileForm.svelte');
		const onCancel = vi.fn();
		render(ProfileForm, {
			props: { onSubmit: vi.fn(), onCancel }
		});
		await fireEvent.click(screen.getByText('Cancel'));
		expect(onCancel).toHaveBeenCalledOnce();
	});

	it('should populate initial values', async () => {
		const { default: ProfileForm } = await import('$lib/components/profiles/ProfileForm.svelte');
		render(ProfileForm, {
			props: {
				initialValues: { id: 1, name: 'Test', description: 'Desc', icon: '🎨', isActive: false, createdAt: '', updatedAt: '' },
				onSubmit: vi.fn(),
				onCancel: vi.fn()
			}
		});
		const input = screen.getByLabelText(/Name/) as HTMLInputElement;
		expect(input.value).toBe('Test');
	});
});

describe('ProfileLibrary Component', () => {
	it('should show empty state', async () => {
		const { default: ProfileLibrary } = await import('$lib/components/profiles/ProfileLibrary.svelte');
		render(ProfileLibrary, { props: {} });
		expect(screen.getByText(/No profiles yet/)).toBeInTheDocument();
	});
});

describe('Profiles index.ts exports', () => {
	it('should export all components', async () => {
		const exports = await import('$lib/components/profiles');
		expect(exports.ProfileCard).toBeDefined();
		expect(exports.ProfileForm).toBeDefined();
		expect(exports.ProfileLibrary).toBeDefined();
	});
});
