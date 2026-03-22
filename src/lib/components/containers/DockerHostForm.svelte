<script lang="ts">
	type Props = {
		onSubmit: (host: any) => void;
		onCancel: () => void;
	};

	let { onSubmit, onCancel }: Props = $props();

	let name = $state('');
	let hostType = $state('local');
	let connectionUri = $state('');
</script>

<form onsubmit={(e) => { e.preventDefault(); onSubmit({ name, hostType, connectionUri }); }}>
	<div class="space-y-4">
		<div>
			<label for="host-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Name *</label>
			<input id="host-name" bind:value={name} required class="input w-full" />
		</div>

		<div>
			<label for="host-type" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Host Type</label>
			<select id="host-type" bind:value={hostType} class="input w-full text-sm">
				<option value="local">Local</option>
				<option value="ssh">SSH</option>
				<option value="tcp">TCP</option>
			</select>
		</div>

		{#if hostType !== 'local'}
			<div>
				<label for="connection-uri" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Connection URI</label>
				<input id="connection-uri" bind:value={connectionUri} class="input w-full" />
			</div>
		{/if}

		<div class="flex justify-end gap-2">
			<button type="button" onclick={onCancel} class="btn btn-secondary">Cancel</button>
			<button type="submit" class="btn btn-primary">Add Host</button>
		</div>
	</div>
</form>
