<script lang="ts">
	import { onMount } from 'svelte';
	import { Header } from '$lib/components/layout';
	import { ProfileLibrary, ProfileForm } from '$lib/components/profiles';
	import { ConfirmDialog } from '$lib/components/shared';
	import { profileLibrary, mcpLibrary, skillLibrary, commandLibrary, subagentLibrary, hookLibrary, notifications } from '$lib/stores';
	import { i18n } from '$lib/i18n';
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
			notifications.success(i18n.t('profile.created'));
		} catch (err) {
			notifications.error(i18n.t('profile.createFailed'));
		}
	}

	async function handleUpdate(values: CreateProfileRequest) {
		if (!editingProfile) return;
		try {
			await profileLibrary.update(editingProfile.id, values);
			editingProfile = null;
			notifications.success(i18n.t('profile.updated'));
		} catch (err) {
			notifications.error(i18n.t('profile.updateFailed'));
		}
	}

	async function handleDelete() {
		if (!deletingProfile) return;
		try {
			await profileLibrary.delete(deletingProfile.id);
			notifications.success(i18n.t('profile.deleted'));
		} catch (err) {
			notifications.error(i18n.t('profile.deleteFailed'));
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
			notifications.success(i18n.t('profile.activated', { name: profile.name }));
		} catch (err) {
			notifications.error(i18n.t('profile.activateFailed'));
		}
	}

	async function handleDeactivate() {
		try {
			await profileLibrary.deactivate();
			notifications.success(i18n.t('profile.deactivated'));
		} catch (err) {
			notifications.error(i18n.t('profile.deactivateFailed'));
		}
	}

	async function handleCapture(profile: Profile) {
		try {
			await profileLibrary.captureFromCurrent(profile.id);
			notifications.success(i18n.t('profile.captured', { name: profile.name }));
		} catch (err) {
			notifications.error(i18n.t('profile.captureFailed'));
		}
	}
</script>

<Header
	title={i18n.t('page.profiles.title')}
	subtitle={i18n.t('page.profiles.subtitle')}
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex justify-end mb-6">
		<button onclick={() => (showAddProfile = true)} class="btn btn-primary">
			<Plus class="w-4 h-4 mr-2" />
			{i18n.t('profile.createProfile')}
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
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">{i18n.t('profile.createProfile')}</h2>
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
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">{i18n.t('profile.editProfile')}</h2>
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
	title={i18n.t('profile.deleteProfile')}
	message={i18n.t('profile.deleteConfirm', { name: deletingProfile?.name ?? '' })}
	confirmText={i18n.t('common.delete')}
	onConfirm={handleDelete}
	onCancel={() => (deletingProfile = null)}
/>
