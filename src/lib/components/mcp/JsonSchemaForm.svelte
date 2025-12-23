<script lang="ts">
	import { ChevronDown, ChevronRight, Plus, Trash2 } from 'lucide-svelte';

	type JsonSchema = {
		type?: string;
		properties?: Record<string, JsonSchema>;
		items?: JsonSchema;
		required?: string[];
		description?: string;
		default?: unknown;
		enum?: unknown[];
		minimum?: number;
		maximum?: number;
		minLength?: number;
		maxLength?: number;
		pattern?: string;
		oneOf?: JsonSchema[];
		anyOf?: JsonSchema[];
		allOf?: JsonSchema[];
		$ref?: string;
	};

	type Props = {
		schema: JsonSchema | null;
		value: Record<string, unknown>;
		onChange: (value: Record<string, unknown>) => void;
		disabled?: boolean;
	};

	let { schema, value, onChange, disabled = false }: Props = $props();

	let expandedSections = $state<Set<string>>(new Set(['root']));

	function toggleSection(path: string) {
		const newSet = new Set(expandedSections);
		if (newSet.has(path)) {
			newSet.delete(path);
		} else {
			newSet.add(path);
		}
		expandedSections = newSet;
	}

	function updateValue(path: string[], newValue: unknown) {
		const result = { ...value };
		let current: Record<string, unknown> = result;

		for (let i = 0; i < path.length - 1; i++) {
			const key = path[i];
			if (current[key] === undefined || current[key] === null) {
				current[key] = {};
			}
			current[key] = { ...(current[key] as Record<string, unknown>) };
			current = current[key] as Record<string, unknown>;
		}

		const lastKey = path[path.length - 1];
		if (newValue === '' || newValue === undefined) {
			delete current[lastKey];
		} else {
			current[lastKey] = newValue;
		}

		onChange(result);
	}

	function getValue(path: string[]): unknown {
		let current: unknown = value;
		for (const key of path) {
			if (current === undefined || current === null) return undefined;
			current = (current as Record<string, unknown>)[key];
		}
		return current;
	}

	function addArrayItem(path: string[], itemSchema: JsonSchema | undefined) {
		const currentArray = (getValue(path) as unknown[]) || [];
		const defaultValue = getDefaultValue(itemSchema);
		updateValue(path, [...currentArray, defaultValue]);
	}

	function removeArrayItem(path: string[], index: number) {
		const currentArray = (getValue(path) as unknown[]) || [];
		updateValue(
			path,
			currentArray.filter((_, i) => i !== index)
		);
	}

	function getDefaultValue(schema: JsonSchema | undefined): unknown {
		if (!schema) return '';
		if (schema.default !== undefined) return schema.default;
		switch (schema.type) {
			case 'string':
				return '';
			case 'number':
			case 'integer':
				return schema.minimum ?? 0;
			case 'boolean':
				return false;
			case 'array':
				return [];
			case 'object':
				return {};
			default:
				return '';
		}
	}

	function isRequired(propName: string, parentSchema: JsonSchema | undefined): boolean {
		return parentSchema?.required?.includes(propName) ?? false;
	}

	function getInputType(schema: JsonSchema): string {
		if (schema.enum) return 'select';
		switch (schema.type) {
			case 'number':
			case 'integer':
				return 'number';
			case 'boolean':
				return 'checkbox';
			default:
				return 'text';
		}
	}
</script>

{#snippet renderField(propName: string, propSchema: JsonSchema, path: string[], parentSchema?: JsonSchema)}
	{@const currentPath = [...path, propName]}
	{@const pathKey = currentPath.join('.')}
	{@const currentValue = getValue(currentPath)}
	{@const required = isRequired(propName, parentSchema)}
	{@const inputType = getInputType(propSchema)}

	<div class="mb-3">
		{#if propSchema.type === 'object' && propSchema.properties}
			<!-- Nested object -->
			<button
				type="button"
				onclick={() => toggleSection(pathKey)}
				class="flex items-center gap-2 w-full text-left py-1 text-sm font-medium text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white"
				{disabled}
			>
				{#if expandedSections.has(pathKey)}
					<ChevronDown class="w-4 h-4" />
				{:else}
					<ChevronRight class="w-4 h-4" />
				{/if}
				<span>{propName}</span>
				{#if required}
					<span class="text-red-500">*</span>
				{/if}
			</button>

			{#if expandedSections.has(pathKey)}
				<div class="ml-4 pl-3 border-l-2 border-gray-200 dark:border-gray-700">
					{#each Object.entries(propSchema.properties) as [childName, childSchema]}
						{@render renderField(childName, childSchema as JsonSchema, currentPath, propSchema)}
					{/each}
				</div>
			{/if}
		{:else if propSchema.type === 'array'}
			<!-- Array field -->
			<div class="space-y-2">
				<div class="flex items-center justify-between">
					<label class="text-sm font-medium text-gray-700 dark:text-gray-300">
						{propName}
						{#if required}
							<span class="text-red-500">*</span>
						{/if}
					</label>
					<button
						type="button"
						onclick={() => addArrayItem(currentPath, propSchema.items)}
						class="flex items-center gap-1 px-2 py-1 text-xs text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded"
						{disabled}
					>
						<Plus class="w-3 h-3" />
						Add
					</button>
				</div>

				{#if propSchema.description}
					<p class="text-xs text-gray-500 dark:text-gray-400">{propSchema.description}</p>
				{/if}

				{#each ((currentValue as unknown[]) || []) as item, index}
					<div class="flex items-start gap-2">
						<div class="flex-1">
							{#if propSchema.items?.type === 'object' && propSchema.items.properties}
								<div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-2">
									{#each Object.entries(propSchema.items.properties) as [childName, childSchema]}
										{@render renderField(
											childName,
											childSchema as JsonSchema,
											[...currentPath, String(index)],
											propSchema.items
										)}
									{/each}
								</div>
							{:else}
								<input
									type={getInputType(propSchema.items || {})}
									value={item ?? ''}
									oninput={(e) =>
										updateValue(
											[...currentPath, String(index)],
											(e.target as HTMLInputElement).value
										)}
									class="w-full px-3 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
									{disabled}
								/>
							{/if}
						</div>
						<button
							type="button"
							onclick={() => removeArrayItem(currentPath, index)}
							class="p-1.5 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
							{disabled}
						>
							<Trash2 class="w-4 h-4" />
						</button>
					</div>
				{/each}
			</div>
		{:else if propSchema.enum}
			<!-- Enum/select field -->
			<label class="block">
				<span class="text-sm font-medium text-gray-700 dark:text-gray-300">
					{propName}
					{#if required}
						<span class="text-red-500">*</span>
					{/if}
				</span>
				{#if propSchema.description}
					<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{propSchema.description}</p>
				{/if}
				<select
					value={currentValue ?? ''}
					onchange={(e) => updateValue(currentPath, (e.target as HTMLSelectElement).value)}
					class="mt-1 w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
					{disabled}
				>
					<option value="">Select...</option>
					{#each propSchema.enum as option}
						<option value={String(option)}>{String(option)}</option>
					{/each}
				</select>
			</label>
		{:else if propSchema.type === 'boolean'}
			<!-- Boolean/checkbox field -->
			<label class="flex items-center gap-2 cursor-pointer">
				<input
					type="checkbox"
					checked={currentValue === true}
					onchange={(e) => updateValue(currentPath, (e.target as HTMLInputElement).checked)}
					class="w-4 h-4 text-blue-600 rounded border-gray-300 dark:border-gray-600 focus:ring-blue-500"
					{disabled}
				/>
				<span class="text-sm font-medium text-gray-700 dark:text-gray-300">
					{propName}
					{#if required}
						<span class="text-red-500">*</span>
					{/if}
				</span>
			</label>
			{#if propSchema.description}
				<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5 ml-6">{propSchema.description}</p>
			{/if}
		{:else}
			<!-- String/number/other field -->
			<label class="block">
				<span class="text-sm font-medium text-gray-700 dark:text-gray-300">
					{propName}
					{#if required}
						<span class="text-red-500">*</span>
					{/if}
				</span>
				{#if propSchema.description}
					<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{propSchema.description}</p>
				{/if}
				<input
					type={inputType}
					value={currentValue ?? ''}
					oninput={(e) => {
						const target = e.target as HTMLInputElement;
						const val = inputType === 'number' ? (target.value ? Number(target.value) : '') : target.value;
						updateValue(currentPath, val);
					}}
					placeholder={propSchema.default !== undefined ? String(propSchema.default) : ''}
					min={propSchema.minimum}
					max={propSchema.maximum}
					minlength={propSchema.minLength}
					maxlength={propSchema.maxLength}
					pattern={propSchema.pattern}
					class="mt-1 w-full px-3 py-2 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed"
					{disabled}
				/>
			</label>
		{/if}
	</div>
{/snippet}

<div class="space-y-2">
	{#if schema?.properties}
		{#each Object.entries(schema.properties) as [propName, propSchema]}
			{@render renderField(propName, propSchema as JsonSchema, [], schema)}
		{/each}
	{:else if schema === null || Object.keys(schema ?? {}).length === 0}
		<p class="text-sm text-gray-500 dark:text-gray-400 italic">
			This tool takes no parameters
		</p>
	{:else}
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
				Arguments (JSON)
			</label>
			<textarea
				value={JSON.stringify(value, null, 2)}
				oninput={(e) => {
					try {
						onChange(JSON.parse((e.target as HTMLTextAreaElement).value));
					} catch {
						// Ignore parse errors while typing
					}
				}}
				rows={5}
				class="w-full px-3 py-2 text-sm font-mono border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
				placeholder={'{}'}
				{disabled}
			></textarea>
		</div>
	{/if}
</div>
