<script lang="ts">
	import type { ClaudeSettings } from '$lib/types';
	import { Save } from 'lucide-svelte';

	type Props = {
		settings: ClaudeSettings;
		onsave: (settings: ClaudeSettings) => void;
	};

	let { settings, onsave }: Props = $props();

	const fields = [
		{
			key: 'apiKeyHelper' as const,
			label: 'API Key Helper',
			description: 'Script that provides the Anthropic API key',
			placeholder: '/path/to/api-key-helper.sh'
		},
		{
			key: 'otelHeadersHelper' as const,
			label: 'OpenTelemetry Headers Helper',
			description: 'Script that provides OpenTelemetry headers',
			placeholder: '/path/to/otel-headers-helper.sh'
		},
		{
			key: 'awsAuthRefresh' as const,
			label: 'AWS Auth Refresh',
			description: 'Script to refresh AWS authentication credentials',
			placeholder: '/path/to/aws-auth-refresh.sh'
		},
		{
			key: 'awsCredentialExport' as const,
			label: 'AWS Credential Export',
			description: 'Script to export AWS credentials to environment',
			placeholder: '/path/to/aws-credential-export.sh'
		}
	];

	let values = $state<Record<string, string>>({
		apiKeyHelper: settings.apiKeyHelper ?? '',
		otelHeadersHelper: settings.otelHeadersHelper ?? '',
		awsAuthRefresh: settings.awsAuthRefresh ?? '',
		awsCredentialExport: settings.awsCredentialExport ?? ''
	});

	$effect(() => {
		values = {
			apiKeyHelper: settings.apiKeyHelper ?? '',
			otelHeadersHelper: settings.otelHeadersHelper ?? '',
			awsAuthRefresh: settings.awsAuthRefresh ?? '',
			awsCredentialExport: settings.awsCredentialExport ?? ''
		};
	});

	function handleSave() {
		onsave({
			...settings,
			apiKeyHelper: values.apiKeyHelper.trim() || undefined,
			otelHeadersHelper: values.otelHeadersHelper.trim() || undefined,
			awsAuthRefresh: values.awsAuthRefresh.trim() || undefined,
			awsCredentialExport: values.awsCredentialExport.trim() || undefined
		});
	}
</script>

<div class="space-y-6">
	<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
		<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">Auth & API Key Helpers</h3>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
			Configure scripts that provide authentication credentials and API keys
		</p>

		<div class="space-y-4">
			{#each fields as field}
				<div>
					<label for={field.key} class="text-sm font-medium text-gray-700 dark:text-gray-300">
						{field.label}
					</label>
					<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
						{field.description}
					</p>
					<input
						id={field.key}
						type="text"
						bind:value={values[field.key]}
						placeholder={field.placeholder}
						class="input w-full"
					/>
				</div>
			{/each}
		</div>
	</div>

	<div class="flex justify-end">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save Auth Helper Settings
		</button>
	</div>
</div>
