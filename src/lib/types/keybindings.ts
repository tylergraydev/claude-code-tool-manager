// ============================================================================
// Keybindings Types & Constants
// ============================================================================

export type KeybindingContext =
	| 'Global'
	| 'Chat'
	| 'Autocomplete'
	| 'Confirm'
	| 'Tabs'
	| 'Transcript'
	| 'HistorySearch'
	| 'Task'
	| 'Theme'
	| 'Help'
	| 'Attachments'
	| 'Footer'
	| 'MessageSelector'
	| 'Diff'
	| 'ModelPicker'
	| 'Select'
	| 'Plugin'
	| 'Permission';

export interface KeybindingContextInfo {
	context: KeybindingContext;
	label: string;
	description: string;
}

export interface KeybindingAction {
	action: string;
	label: string;
	description: string;
	context: KeybindingContext;
	defaultKeys: string[];
}

export interface ContextBindings {
	context: string;
	bindings: Record<string, string | null>;
}

export interface KeybindingsFile {
	schema?: string;
	bindings: ContextBindings[];
}

export interface MergedBinding {
	action: string;
	label: string;
	description: string;
	context: KeybindingContext;
	defaultKeys: string[];
	currentKeys: string[];
	isModified: boolean;
	unboundKeys: string[];
	addedKeys: string[];
}

export interface KeyConflict {
	key: string;
	context: KeybindingContext;
	existingAction: string;
	existingActionLabel: string;
}

// ============================================================================
// Context Definitions (18 contexts)
// ============================================================================

export const KEYBINDING_CONTEXTS: KeybindingContextInfo[] = [
	{
		context: 'Global',
		label: 'Global',
		description: 'Application-wide keybindings available in all contexts'
	},
	{
		context: 'Chat',
		label: 'Chat',
		description: 'Keybindings active in the chat input area'
	},
	{
		context: 'Autocomplete',
		label: 'Autocomplete',
		description: 'Keybindings for autocomplete suggestion navigation'
	},
	{
		context: 'Confirm',
		label: 'Confirm',
		description: 'Keybindings for confirmation dialogs and prompts'
	},
	{
		context: 'Tabs',
		label: 'Tabs',
		description: 'Keybindings for switching between tabs'
	},
	{
		context: 'Transcript',
		label: 'Transcript',
		description: 'Keybindings for transcript viewing and navigation'
	},
	{
		context: 'HistorySearch',
		label: 'History Search',
		description: 'Keybindings for searching through command history'
	},
	{
		context: 'Task',
		label: 'Task',
		description: 'Keybindings for task management actions'
	},
	{
		context: 'Theme',
		label: 'Theme',
		description: 'Keybindings for theme-related toggles'
	},
	{
		context: 'Help',
		label: 'Help',
		description: 'Keybindings for help screen actions'
	},
	{
		context: 'Attachments',
		label: 'Attachments',
		description: 'Keybindings for managing file attachments'
	},
	{
		context: 'Footer',
		label: 'Footer',
		description: 'Keybindings for footer navigation and selection'
	},
	{
		context: 'MessageSelector',
		label: 'Message Selector',
		description: 'Keybindings for navigating and selecting messages'
	},
	{
		context: 'Diff',
		label: 'Diff',
		description: 'Keybindings for diff viewing and navigation'
	},
	{
		context: 'ModelPicker',
		label: 'Model Picker',
		description: 'Keybindings for model selection and effort control'
	},
	{
		context: 'Select',
		label: 'Select',
		description: 'Keybindings for generic selection lists'
	},
	{
		context: 'Plugin',
		label: 'Plugin',
		description: 'Keybindings for plugin management actions'
	},
	{
		context: 'Permission',
		label: 'Permission',
		description: 'Keybindings for permission dialog actions'
	}
];

// ============================================================================
// Action Definitions (69 actions)
// ============================================================================

export const KEYBINDING_ACTIONS: KeybindingAction[] = [
	// Global context
	{
		action: 'app:interrupt',
		label: 'Interrupt',
		description: 'Interrupt the current operation',
		context: 'Global',
		defaultKeys: ['escape']
	},
	{
		action: 'app:exit',
		label: 'Exit',
		description: 'Exit Claude Code',
		context: 'Global',
		defaultKeys: ['ctrl+c']
	},
	{
		action: 'app:toggleTodos',
		label: 'Toggle Todos',
		description: 'Toggle the todo list panel',
		context: 'Global',
		defaultKeys: []
	},
	{
		action: 'app:toggleTranscript',
		label: 'Toggle Transcript',
		description: 'Toggle the transcript view',
		context: 'Global',
		defaultKeys: []
	},
	{
		action: 'app:toggleTeammatePreview',
		label: 'Toggle Teammate Preview',
		description: 'Toggle the teammate preview panel',
		context: 'Global',
		defaultKeys: []
	},
	{
		action: 'history:search',
		label: 'Search History',
		description: 'Open the history search',
		context: 'Global',
		defaultKeys: ['ctrl+r']
	},
	{
		action: 'history:previous',
		label: 'Previous History',
		description: 'Navigate to the previous history entry',
		context: 'Global',
		defaultKeys: ['up']
	},
	{
		action: 'history:next',
		label: 'Next History',
		description: 'Navigate to the next history entry',
		context: 'Global',
		defaultKeys: ['down']
	},

	// Chat context
	{
		action: 'chat:cancel',
		label: 'Cancel',
		description: 'Cancel the current chat operation',
		context: 'Chat',
		defaultKeys: []
	},
	{
		action: 'chat:cycleMode',
		label: 'Cycle Mode',
		description: 'Cycle through chat input modes',
		context: 'Chat',
		defaultKeys: ['ctrl+k']
	},
	{
		action: 'chat:modelPicker',
		label: 'Model Picker',
		description: 'Open the model picker',
		context: 'Chat',
		defaultKeys: []
	},
	{
		action: 'chat:thinkingToggle',
		label: 'Toggle Thinking',
		description: 'Toggle extended thinking mode',
		context: 'Chat',
		defaultKeys: []
	},
	{
		action: 'chat:submit',
		label: 'Submit',
		description: 'Send the current message',
		context: 'Chat',
		defaultKeys: ['enter']
	},
	{
		action: 'chat:undo',
		label: 'Undo',
		description: 'Undo the last message',
		context: 'Chat',
		defaultKeys: []
	},
	{
		action: 'chat:externalEditor',
		label: 'External Editor',
		description: 'Open an external editor for the message',
		context: 'Chat',
		defaultKeys: ['ctrl+e']
	},
	{
		action: 'chat:stash',
		label: 'Stash',
		description: 'Stash the current message',
		context: 'Chat',
		defaultKeys: []
	},
	{
		action: 'chat:imagePaste',
		label: 'Image Paste',
		description: 'Paste an image from clipboard',
		context: 'Chat',
		defaultKeys: ['ctrl+v']
	},

	// Autocomplete context
	{
		action: 'autocomplete:accept',
		label: 'Accept',
		description: 'Accept the current suggestion',
		context: 'Autocomplete',
		defaultKeys: ['tab']
	},
	{
		action: 'autocomplete:dismiss',
		label: 'Dismiss',
		description: 'Dismiss the autocomplete menu',
		context: 'Autocomplete',
		defaultKeys: ['escape']
	},
	{
		action: 'autocomplete:previous',
		label: 'Previous',
		description: 'Select the previous suggestion',
		context: 'Autocomplete',
		defaultKeys: ['up']
	},
	{
		action: 'autocomplete:next',
		label: 'Next',
		description: 'Select the next suggestion',
		context: 'Autocomplete',
		defaultKeys: ['down']
	},

	// Confirm context
	{
		action: 'confirm:yes',
		label: 'Yes',
		description: 'Confirm the action',
		context: 'Confirm',
		defaultKeys: ['y']
	},
	{
		action: 'confirm:no',
		label: 'No',
		description: 'Deny the action',
		context: 'Confirm',
		defaultKeys: ['n']
	},
	{
		action: 'confirm:previous',
		label: 'Previous',
		description: 'Move to the previous option',
		context: 'Confirm',
		defaultKeys: ['up']
	},
	{
		action: 'confirm:next',
		label: 'Next',
		description: 'Move to the next option',
		context: 'Confirm',
		defaultKeys: ['down']
	},
	{
		action: 'confirm:nextField',
		label: 'Next Field',
		description: 'Move to the next input field',
		context: 'Confirm',
		defaultKeys: ['tab']
	},
	{
		action: 'confirm:previousField',
		label: 'Previous Field',
		description: 'Move to the previous input field',
		context: 'Confirm',
		defaultKeys: ['shift+tab']
	},
	{
		action: 'confirm:cycleMode',
		label: 'Cycle Mode',
		description: 'Cycle through confirmation modes',
		context: 'Confirm',
		defaultKeys: []
	},
	{
		action: 'confirm:toggleExplanation',
		label: 'Toggle Explanation',
		description: 'Show or hide the explanation',
		context: 'Confirm',
		defaultKeys: []
	},

	// Tabs context
	{
		action: 'tabs:next',
		label: 'Next Tab',
		description: 'Switch to the next tab',
		context: 'Tabs',
		defaultKeys: ['tab']
	},
	{
		action: 'tabs:previous',
		label: 'Previous Tab',
		description: 'Switch to the previous tab',
		context: 'Tabs',
		defaultKeys: ['shift+tab']
	},

	// Transcript context
	{
		action: 'transcript:toggleShowAll',
		label: 'Toggle Show All',
		description: 'Toggle between showing all and filtered messages',
		context: 'Transcript',
		defaultKeys: []
	},
	{
		action: 'transcript:exit',
		label: 'Exit Transcript',
		description: 'Close the transcript view',
		context: 'Transcript',
		defaultKeys: ['escape']
	},

	// History Search context
	{
		action: 'historySearch:next',
		label: 'Next Result',
		description: 'Jump to the next search result',
		context: 'HistorySearch',
		defaultKeys: ['ctrl+r']
	},
	{
		action: 'historySearch:accept',
		label: 'Accept Result',
		description: 'Accept the current search result',
		context: 'HistorySearch',
		defaultKeys: ['enter']
	},
	{
		action: 'historySearch:cancel',
		label: 'Cancel Search',
		description: 'Cancel the history search',
		context: 'HistorySearch',
		defaultKeys: ['escape']
	},
	{
		action: 'historySearch:execute',
		label: 'Execute',
		description: 'Execute the selected history entry',
		context: 'HistorySearch',
		defaultKeys: []
	},

	// Task context
	{
		action: 'task:background',
		label: 'Background Task',
		description: 'Send the current task to the background',
		context: 'Task',
		defaultKeys: []
	},

	// Theme context
	{
		action: 'theme:toggleSyntaxHighlighting',
		label: 'Toggle Syntax Highlighting',
		description: 'Toggle syntax highlighting on or off',
		context: 'Theme',
		defaultKeys: []
	},

	// Help context
	{
		action: 'help:dismiss',
		label: 'Dismiss Help',
		description: 'Close the help screen',
		context: 'Help',
		defaultKeys: ['escape']
	},

	// Attachments context
	{
		action: 'attachments:next',
		label: 'Next Attachment',
		description: 'Select the next attachment',
		context: 'Attachments',
		defaultKeys: ['right']
	},
	{
		action: 'attachments:previous',
		label: 'Previous Attachment',
		description: 'Select the previous attachment',
		context: 'Attachments',
		defaultKeys: ['left']
	},
	{
		action: 'attachments:remove',
		label: 'Remove Attachment',
		description: 'Remove the selected attachment',
		context: 'Attachments',
		defaultKeys: ['backspace']
	},
	{
		action: 'attachments:exit',
		label: 'Exit Attachments',
		description: 'Close the attachment selector',
		context: 'Attachments',
		defaultKeys: ['escape']
	},

	// Footer context
	{
		action: 'footer:next',
		label: 'Next Item',
		description: 'Select the next footer item',
		context: 'Footer',
		defaultKeys: ['right']
	},
	{
		action: 'footer:previous',
		label: 'Previous Item',
		description: 'Select the previous footer item',
		context: 'Footer',
		defaultKeys: ['left']
	},
	{
		action: 'footer:openSelected',
		label: 'Open Selected',
		description: 'Open the currently selected footer item',
		context: 'Footer',
		defaultKeys: ['enter']
	},
	{
		action: 'footer:clearSelection',
		label: 'Clear Selection',
		description: 'Clear the footer selection',
		context: 'Footer',
		defaultKeys: ['escape']
	},

	// Message Selector context
	{
		action: 'messageSelector:up',
		label: 'Move Up',
		description: 'Select the previous message',
		context: 'MessageSelector',
		defaultKeys: ['up']
	},
	{
		action: 'messageSelector:down',
		label: 'Move Down',
		description: 'Select the next message',
		context: 'MessageSelector',
		defaultKeys: ['down']
	},
	{
		action: 'messageSelector:top',
		label: 'Move to Top',
		description: 'Jump to the first message',
		context: 'MessageSelector',
		defaultKeys: []
	},
	{
		action: 'messageSelector:bottom',
		label: 'Move to Bottom',
		description: 'Jump to the last message',
		context: 'MessageSelector',
		defaultKeys: []
	},
	{
		action: 'messageSelector:select',
		label: 'Select Message',
		description: 'Select the highlighted message',
		context: 'MessageSelector',
		defaultKeys: ['enter']
	},

	// Diff context
	{
		action: 'diff:dismiss',
		label: 'Dismiss Diff',
		description: 'Close the diff view',
		context: 'Diff',
		defaultKeys: ['escape']
	},
	{
		action: 'diff:previousSource',
		label: 'Previous Source',
		description: 'Navigate to the previous diff source',
		context: 'Diff',
		defaultKeys: []
	},
	{
		action: 'diff:nextSource',
		label: 'Next Source',
		description: 'Navigate to the next diff source',
		context: 'Diff',
		defaultKeys: []
	},
	{
		action: 'diff:back',
		label: 'Back',
		description: 'Go back to the previous diff view',
		context: 'Diff',
		defaultKeys: []
	},
	{
		action: 'diff:viewDetails',
		label: 'View Details',
		description: 'Show detailed diff information',
		context: 'Diff',
		defaultKeys: ['enter']
	},
	{
		action: 'diff:previousFile',
		label: 'Previous File',
		description: 'Navigate to the previous file in the diff',
		context: 'Diff',
		defaultKeys: ['up']
	},
	{
		action: 'diff:nextFile',
		label: 'Next File',
		description: 'Navigate to the next file in the diff',
		context: 'Diff',
		defaultKeys: ['down']
	},

	// Model Picker context
	{
		action: 'modelPicker:decreaseEffort',
		label: 'Decrease Effort',
		description: 'Lower the model effort level',
		context: 'ModelPicker',
		defaultKeys: ['left']
	},
	{
		action: 'modelPicker:increaseEffort',
		label: 'Increase Effort',
		description: 'Raise the model effort level',
		context: 'ModelPicker',
		defaultKeys: ['right']
	},

	// Select context
	{
		action: 'select:next',
		label: 'Next Option',
		description: 'Select the next option in the list',
		context: 'Select',
		defaultKeys: ['down']
	},
	{
		action: 'select:previous',
		label: 'Previous Option',
		description: 'Select the previous option in the list',
		context: 'Select',
		defaultKeys: ['up']
	},
	{
		action: 'select:accept',
		label: 'Accept',
		description: 'Accept the current selection',
		context: 'Select',
		defaultKeys: ['enter']
	},
	{
		action: 'select:cancel',
		label: 'Cancel',
		description: 'Cancel the selection',
		context: 'Select',
		defaultKeys: ['escape']
	},

	// Plugin context
	{
		action: 'plugin:toggle',
		label: 'Toggle Plugin',
		description: 'Enable or disable the selected plugin',
		context: 'Plugin',
		defaultKeys: []
	},
	{
		action: 'plugin:install',
		label: 'Install Plugin',
		description: 'Install the selected plugin',
		context: 'Plugin',
		defaultKeys: []
	},

	// Permission context
	{
		action: 'permission:toggleDebug',
		label: 'Toggle Debug',
		description: 'Toggle debug info in permission prompts',
		context: 'Permission',
		defaultKeys: []
	},
	{
		action: 'settings:search',
		label: 'Search Settings',
		description: 'Search through settings',
		context: 'Permission',
		defaultKeys: []
	},
	{
		action: 'settings:retry',
		label: 'Retry Settings',
		description: 'Retry the current settings operation',
		context: 'Permission',
		defaultKeys: []
	}
];

// ============================================================================
// Reserved & Conflict Keys
// ============================================================================

/** Keys that cannot be rebound — critical terminal signals */
export const RESERVED_KEYS = ['ctrl+c', 'ctrl+d'];

/** Keys that will generate a warning — they conflict with common terminal bindings */
export const TERMINAL_CONFLICT_KEYS = ['ctrl+z', 'ctrl+b', 'ctrl+a'];

// ============================================================================
// Helpers
// ============================================================================

/** Format a key combo for display: "ctrl+shift+k" → "Ctrl+Shift+K" */
export function formatKeystroke(key: string): string {
	return key
		.split('+')
		.map((part) => {
			if (part === 'ctrl') return 'Ctrl';
			if (part === 'alt') return 'Alt';
			if (part === 'shift') return 'Shift';
			if (part === 'meta') return 'Meta';
			if (part === 'escape') return 'Esc';
			if (part === 'enter') return 'Enter';
			if (part === 'tab') return 'Tab';
			if (part === 'space') return 'Space';
			if (part === 'backspace') return 'Backspace';
			if (part === 'delete') return 'Delete';
			if (part === 'up') return 'Up';
			if (part === 'down') return 'Down';
			if (part === 'left') return 'Left';
			if (part === 'right') return 'Right';
			if (part === 'home') return 'Home';
			if (part === 'end') return 'End';
			if (part === 'pageup') return 'PageUp';
			if (part === 'pagedown') return 'PageDown';
			if (part.length === 1) return part.toUpperCase();
			return part.charAt(0).toUpperCase() + part.slice(1);
		})
		.join('+');
}

/** Get actions for a specific context */
export function getActionsForContext(context: KeybindingContext): KeybindingAction[] {
	return KEYBINDING_ACTIONS.filter((a) => a.context === context);
}
