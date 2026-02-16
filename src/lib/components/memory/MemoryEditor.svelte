<script lang="ts">
	type Props = {
		content: string;
		onchange: (content: string) => void;
	};

	let { content, onchange }: Props = $props();

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Tab') {
			e.preventDefault();
			const textarea = e.target as HTMLTextAreaElement;
			const start = textarea.selectionStart;
			const end = textarea.selectionEnd;
			const value = textarea.value;
			const newValue = value.substring(0, start) + '\t' + value.substring(end);
			textarea.value = newValue;
			textarea.selectionStart = textarea.selectionEnd = start + 1;
			onchange(newValue);
		}
	}

	function handleInput(e: Event) {
		const textarea = e.target as HTMLTextAreaElement;
		onchange(textarea.value);
	}
</script>

<textarea
	value={content}
	oninput={handleInput}
	onkeydown={handleKeydown}
	class="w-full h-full min-h-[400px] p-4 font-mono text-sm
		bg-white dark:bg-gray-900
		text-gray-900 dark:text-gray-100
		border border-gray-200 dark:border-gray-700
		rounded-lg resize-none
		focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent
		placeholder:text-gray-400 dark:placeholder:text-gray-500"
	placeholder="# CLAUDE.md&#10;&#10;Write your instructions for Claude here...&#10;&#10;- Project guidelines&#10;- Code conventions&#10;- Important context"
	spellcheck="false"
></textarea>
