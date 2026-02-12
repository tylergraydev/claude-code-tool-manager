<script lang="ts">
	import { onMount } from 'svelte';
	import { Header } from '$lib/components/layout';
	import { ProfileLibrary, ProfileForm } from '$lib/components/profiles';
	import { ConfirmDialog } from '$lib/components/shared';
	import { profileLibrary, mcpLibrary, skillLibrary, commandLibrary, subagentLibrary, hookLibrary, notifications } from '$lib/stores';
	import type { Profile, CreateProfileRequest } from '$lib/types';
	import { Plus } from 'lucide-svelte';

	let showAddProfile = $state(false);
	let editingProfile = $state<Profile | null>(null);
	let deletingProfile = $state<Profile | null>(null);

	onMount(async () => {
		await profileLibrary.load();
		await profileLibrary.loadActiveProfile();
	});

	async function handleCreate(values: CreateProfileRequest) {
		try {
			await profileLibrary.create(values);
			showAddProfile = false;
			notifications.success('Profile created');
		} catch (err) {
			notifications.error('Failed to create profile');
		}
	}

	async function handleUpdate(values: CreateProfileRequest) {
		if (!editingProfile) return;
		try {
			await profileLibrary.update(editingProfile.id, values);
			editingProfile = null;
			notifications.success('Profile updated');
		} catch (err) {
			notifications.error('Failed to update profile');
		}
	}

	async function handleDelete() {
		if (!deletingProfile) return;
		try {
			await profileLibrary.delete(deletingProfile.id);
			notifications.success('Profile deleted');
		} catch (err) {
			notifications.error('Failed to delete profile');
		} finally {
			deletingProfile = null;
		}
	}

	async function handleActivate(profile: Profile) {
		try {
			await profileLibrary.activate(profile.id);
			// Reload other stores so UI reflects the new global state
			await Promise.all([
				mcpLibrary.load(),
				skillLibrary.load(),
				commandLibrary.load(),
				subagentLibrary.load(),
				hookLibrary.load(),
				hookLibrary.loadGlobalHooks()
			]);
			notifications.success(`Profile "${profile.name}" activated`);
		} catch (err) {
			notifications.error('Failed to activate profile');
		}
	}

	async function handleDeactivate() {
		try {
			await profileLibrary.deactivate();
			notifications.success('Profile deactivated');
		} catch (err) {
			notifications.error('Failed to deactivate profile');
		}
	}

	async function handleCapture(profile: Profile) {
		try {
			await profileLibrary.captureFromCurrent(profile.id);
			notifications.success(`Captured current config into "${profile.name}"`);
		} catch (err) {
			notifications.error('Failed to capture configuration');
		}
	}
</script>

<Header
	title="Configuration Profiles"
	subtitle="Save and switch between different sets of globally-enabled tools"
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex justify-end mb-6">
		<button onclick={() => (showAddProfile = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			Create Profile
		</button>
	</div>

	<ProfileLibrary
		onActivate={handleActivate}
		onDeactivate={handleDeactivate}
		onCapture={handleCapture}
		onEdit={(profile) => (editingProfile = profile)}
		onDelete={(profile) => (deletingProfile = profile)}
	/>
</div>

<!-- Add Profile Modal -->
{#if showAddProfile}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-md w-full mx-4">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Create Profile</h2>
				<ProfileForm
					onSubmit={handleCreate}
					onCancel={() => (showAddProfile = false)}
				/>
			</div>
		</div>
	</div>
{/if}

<!-- Edit Profile Modal -->
{#if editingProfile}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-md w-full mx-4">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Edit Profile</h2>
				<ProfileForm
					initialValues={editingProfile}
					onSubmit={handleUpdate}
					onCancel={() => (editingProfile = null)}
				/>
			</div>
		</div>
	</div>
{/if}

<ConfirmDialog
	open={!!deletingProfile}
	title="Delete Profile"
	message="Are you sure you want to delete '{deletingProfile?.name}'? This cannot be undone."
	confirmText="Delete"
	onConfirm={handleDelete}
	onCancel={() => (deletingProfile = null)}
/>
