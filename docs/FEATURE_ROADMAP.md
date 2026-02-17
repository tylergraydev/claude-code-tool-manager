# Feature Roadmap - Claude Code Tool Manager

## Research Source

Based on full review of Claude Code documentation (reviewed 2026-02-16):
- [Settings](https://code.claude.com/docs/en/settings)
- [Keybindings](https://code.claude.com/docs/en/keybindings)

---

## Currently Implemented

| Feature | Route | Notes |
|---------|-------|-------|
| MCPs Library (stdio/sse/http) | `/library` | Full CRUD + gateway integration |
| Projects | `/projects` | Per-project MCP/skill/hook/command/subagent assignments |
| Skills (auto-invoke) | `/skills` | Markdown content + file attachments |
| Slash Commands | `/commands` | Markdown content + allowed tools |
| Sub-Agents | `/subagents` | System prompt + tools + permission mode |
| Hooks | `/hooks` | PreToolUse, PostToolUse, Notification, Stop, SubAgentStop |
| Profiles | `/profiles` | Snapshot/restore global config sets |
| Status Line Builder | `/statusline` | Custom builder + premade gallery |
| Spinner Verbs | `/spinnerverbs` | Append/replace mode |
| Marketplace | `/marketplace` | Browse/import from GitHub repos |
| Global Settings | `/settings` | Editor sync, gateway, MCP server |
| Multi-Editor Sync | — | Claude Code, OpenCode, Codex, Copilot, Cursor, Gemini |

---

## Tier 1 - High Impact, Natural Fit

### 1. Permissions Manager
- **Settings key:** `permissions.allow`, `permissions.ask`, `permissions.deny`
- **Description:** Visual rule builder for tool permission rules with glob pattern support
- **Details:**
  - Rule format: `Tool` or `Tool(specifier)` with glob patterns
  - Tools: `Bash`, `Read`, `Edit`, `Write`, `WebFetch`, `MCP`, `Task`
  - Specifiers: `Bash(npm run *)`, `Read(./.env)`, `WebFetch(domain:example.com)`, `Edit(./*.json)`
  - Three categories: allow (auto-approve), ask (require confirmation), deny (block)
  - Scope: user (`~/.claude/settings.json`), project (`.claude/settings.json`), local (`.claude/settings.local.json`)
- **UI concept:** Drag-and-drop rule list with pattern builder, scope selector, test/preview

### 2. Memory File Editor (CLAUDE.md)
- **Files:** `~/.claude/CLAUDE.md`, `CLAUDE.md`, `.claude/CLAUDE.md`, `CLAUDE.local.md`
- **Description:** View, edit, and manage memory files across all scopes
- **Details:**
  - User-level memory: `~/.claude/CLAUDE.md`
  - Project-level memory: `CLAUDE.md` or `.claude/CLAUDE.md` (committed to git)
  - Local-only memory: `CLAUDE.local.md` (gitignored)
  - Files are loaded automatically at startup and provide persistent instructions/context
- **UI concept:** Multi-tab editor with scope tabs, markdown preview, per-project file management

### 3. Model & Output Configuration
- **Settings keys:** `model`, `availableModels`, `outputStyle`, `language`, `alwaysThinkingEnabled`
- **Description:** Configure default model, restrict model selection, set output preferences
- **Details:**
  - `model` — Default model ID (e.g., `"claude-sonnet-4-5-20250929"`)
  - `availableModels` — Array to restrict selectable models (e.g., `["sonnet", "haiku"]`)
  - `outputStyle` — Adjust system prompt output style (e.g., `"Explanatory"`)
  - `language` — Response language (e.g., `"japanese"`, `"spanish"`, `"french"`)
  - `alwaysThinkingEnabled` — Enable extended thinking by default
- **UI concept:** Form with model dropdown, language selector, output style picker, thinking toggle

### 4. Attribution Editor
- **Settings key:** `attribution.commit`, `attribution.pr`
- **Description:** Customize git commit and pull request attribution text
- **Details:**
  - `attribution.commit` — Text appended to git commits (e.g., `"Co-Authored-By: Claude..."`)
  - `attribution.pr` — Text appended to PR descriptions
  - Set to empty string to hide attribution entirely
  - Replaces deprecated `includeCoAuthoredBy` setting
- **UI concept:** Text editors with preview of how commit/PR messages will look

---

## Tier 2 - Good Additions

### 5. Sandbox Configuration
- **Settings key:** `sandbox.*`
- **Description:** Full sandbox configuration for bash command isolation
- **Details:**
  - `sandbox.enabled` — Enable bash sandboxing (macOS, Linux, WSL2)
  - `sandbox.autoAllowBashIfSandboxed` — Auto-approve commands inside sandbox
  - `sandbox.excludedCommands` — Commands that run outside sandbox (e.g., `["git", "docker"]`)
  - `sandbox.allowUnsandboxedCommands` — Allow `dangerouslyDisableSandbox` parameter
  - `sandbox.enableWeakerNestedSandbox` — Weaker sandbox for Docker environments
  - `sandbox.network.allowUnixSockets` — Accessible Unix socket paths
  - `sandbox.network.allowAllUnixSockets` — Allow all Unix sockets
  - `sandbox.network.allowLocalBinding` — Bind to localhost ports (macOS only)
  - `sandbox.network.allowedDomains` — Allowed outbound domains with wildcard support
  - `sandbox.network.httpProxyPort` / `sandbox.network.socksProxyPort` — Proxy ports
- **UI concept:** Toggle panel with network config section, domain allowlist editor, excluded commands list

### 6. Plugin Manager
- **Settings keys:** `enabledPlugins`, `extraKnownMarketplaces`
- **Description:** Enable/disable plugins and manage marketplace sources
- **Details:**
  - `enabledPlugins` — Map of `"plugin-name@marketplace-name": true/false`
  - `extraKnownMarketplaces` — Define additional marketplace sources
  - Marketplace source types: GitHub, Git, URL, NPM, File, Directory, Host pattern
  - Each source type has specific config (repo, url, package, path, etc.)
- **UI concept:** Plugin list with toggles, marketplace source manager with type-specific forms

### 7. Environment Variables Manager
- **Settings key:** `env`
- **Description:** GUI for managing environment variables passed to all Claude sessions
- **Details:**
  - `env` setting in `settings.json` — Key-value pairs set for all sessions
  - Claude Code also supports 50+ environment variables for configuration:
    - Auth/API: `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_CUSTOM_HEADERS`, `ANTHROPIC_MODEL`
    - Cloud providers: `CLAUDE_CODE_USE_BEDROCK`, `CLAUDE_CODE_USE_VERTEX`, `CLAUDE_CODE_USE_FOUNDRY`
    - Shell: `CLAUDE_CODE_SHELL`, `CLAUDE_CODE_SHELL_PREFIX`, `BASH_DEFAULT_TIMEOUT_MS`
    - Model: `CLAUDE_CODE_EFFORT_LEVEL`, `CLAUDE_CODE_MAX_OUTPUT_TOKENS`, `MAX_THINKING_TOKENS`
    - Telemetry: `CLAUDE_CODE_ENABLE_TELEMETRY`, `DISABLE_TELEMETRY`, `DISABLE_ERROR_REPORTING`
    - UI: `CLAUDE_CODE_HIDE_ACCOUNT_INFO`, `CLAUDE_CODE_DISABLE_TERMINAL_TITLE`
    - MCP: `MCP_TIMEOUT`, `MCP_TOOL_TIMEOUT`, `MAX_MCP_OUTPUT_TOKENS`, `ENABLE_TOOL_SEARCH`
    - Memory: `CLAUDE_CODE_DISABLE_AUTO_MEMORY`, `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE`
    - Proxy: `HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY`
- **UI concept:** Key-value editor with categorized sections, known-variable autocomplete, descriptions

### 8. UI Toggles Page
- **Description:** Centralized page for all boolean/simple settings
- **Settings:**
  - `showTurnDuration` — Show timing after responses
  - `spinnerTipsEnabled` — Show tips while Claude works (default: `true`)
  - `terminalProgressBarEnabled` — Enable terminal progress bar (default: `true`)
  - `prefersReducedMotion` — Reduce UI animations (default: `false`)
  - `respectGitignore` — Respect `.gitignore` in file picker (default: `true`)
- **UI concept:** Clean toggle list with descriptions, grouped by category

---

## Tier 3 - Nice to Have

### 9. Keybindings Editor
- **File:** `~/.claude/keybindings.json`
- **Schema:** `https://www.schemastore.org/claude-code-keybindings.json`
- **Open via:** `/keybindings` slash command
- **Hot reload:** Changes auto-detected, no restart needed
- **Description:** Visual editor for custom keyboard shortcuts and chord bindings
- **File format:**
  ```json
  {
    "$schema": "https://www.schemastore.org/claude-code-keybindings.json",
    "bindings": [
      {
        "context": "Chat",
        "bindings": {
          "ctrl+e": "chat:externalEditor",
          "ctrl+u": null
        }
      }
    ]
  }
  ```
- **17 Contexts:**
  - `Global` — Everywhere in the app
  - `Chat` — Main chat input area
  - `Autocomplete` — Autocomplete menu open
  - `Settings` — Settings menu (escape-only dismiss)
  - `Confirmation` — Permission and confirmation dialogs
  - `Tabs` — Tab navigation components
  - `Help` — Help menu visible
  - `Transcript` — Transcript viewer
  - `HistorySearch` — History search mode (Ctrl+R)
  - `Task` — Background task running
  - `ThemePicker` — Theme picker dialog
  - `Attachments` — Image/attachment bar navigation
  - `Footer` — Footer indicator navigation (tasks, teams, diff)
  - `MessageSelector` — Rewind and summarize dialog
  - `DiffDialog` — Diff viewer navigation
  - `ModelPicker` — Model picker effort level
  - `Select` — Generic select/list components
  - `Plugin` — Plugin dialog (browse, discover, manage)
- **~60 Actions (namespace:action format):**
  - **App (Global):** `app:interrupt` (Ctrl+C), `app:exit` (Ctrl+D), `app:toggleTodos` (Ctrl+T), `app:toggleTranscript` (Ctrl+O)
  - **History:** `history:search` (Ctrl+R), `history:previous` (Up), `history:next` (Down)
  - **Chat:** `chat:cancel` (Escape), `chat:cycleMode` (Shift+Tab), `chat:modelPicker` (Cmd+P/Meta+P), `chat:thinkingToggle` (Cmd+T/Meta+T), `chat:submit` (Enter), `chat:undo` (Ctrl+\_), `chat:externalEditor` (Ctrl+G), `chat:stash` (Ctrl+S), `chat:imagePaste` (Ctrl+V / Alt+V on Windows)
  - **Autocomplete:** `autocomplete:accept` (Tab), `autocomplete:dismiss` (Escape), `autocomplete:previous` (Up), `autocomplete:next` (Down)
  - **Confirmation:** `confirm:yes` (Y/Enter), `confirm:no` (N/Escape), `confirm:previous` (Up), `confirm:next` (Down), `confirm:nextField` (Tab), `confirm:previousField` (unbound), `confirm:cycleMode` (Shift+Tab), `confirm:toggleExplanation` (Ctrl+E)
  - **Permission:** `permission:toggleDebug` (Ctrl+D)
  - **Transcript:** `transcript:toggleShowAll` (Ctrl+E), `transcript:exit` (Ctrl+C/Escape)
  - **HistorySearch:** `historySearch:next` (Ctrl+R), `historySearch:accept` (Escape/Tab), `historySearch:cancel` (Ctrl+C), `historySearch:execute` (Enter)
  - **Task:** `task:background` (Ctrl+B)
  - **Theme:** `theme:toggleSyntaxHighlighting` (Ctrl+T)
  - **Help:** `help:dismiss` (Escape)
  - **Tabs:** `tabs:next` (Tab/Right), `tabs:previous` (Shift+Tab/Left)
  - **Attachments:** `attachments:next` (Right), `attachments:previous` (Left), `attachments:remove` (Backspace/Delete), `attachments:exit` (Down/Escape)
  - **Footer:** `footer:next` (Right), `footer:previous` (Left), `footer:openSelected` (Enter), `footer:clearSelection` (Escape)
  - **MessageSelector:** `messageSelector:up` (Up/K), `messageSelector:down` (Down/J), `messageSelector:top` (Ctrl+Up/Shift+Up/Meta+Up/Shift+K), `messageSelector:bottom` (Ctrl+Down/Shift+Down/Meta+Down/Shift+J), `messageSelector:select` (Enter)
  - **Diff:** `diff:dismiss` (Escape), `diff:previousSource` (Left), `diff:nextSource` (Right), `diff:previousFile` (Up), `diff:nextFile` (Down), `diff:viewDetails` (Enter), `diff:back` (context-specific)
  - **ModelPicker:** `modelPicker:decreaseEffort` (Left), `modelPicker:increaseEffort` (Right)
  - **Select:** `select:next` (Down/J/Ctrl+N), `select:previous` (Up/K/Ctrl+P), `select:accept` (Enter), `select:cancel` (Escape)
  - **Plugin:** `plugin:toggle` (Space), `plugin:install` (I)
  - **Settings:** `settings:search` (/), `settings:retry` (R)
- **Keystroke syntax:**
  - Modifiers: `ctrl`/`control`, `alt`/`opt`/`option`, `shift`, `meta`/`cmd`/`command` — joined with `+`
  - Chords: Space-separated sequences (e.g., `ctrl+k ctrl+s`)
  - Uppercase letters: Standalone `K` = `shift+k`, but `ctrl+K` = `ctrl+k` (no implicit shift with modifiers)
  - Special keys: `escape`/`esc`, `enter`/`return`, `tab`, `space`, `up`/`down`/`left`/`right`, `backspace`, `delete`
  - Unbind: Set action to `null`
  - Reserved (cannot rebind): `Ctrl+C`, `Ctrl+D`
  - Terminal conflicts: `Ctrl+B` (tmux prefix), `Ctrl+A` (GNU screen), `Ctrl+Z` (SIGTSTP)
- **Validation:** Claude Code validates keybindings and warns on parse errors, invalid contexts, reserved conflicts, terminal conflicts, and duplicates. Run `/doctor` to check.
- **Vim mode interaction:** Keybindings and vim mode are independent layers. Vim handles text input level; keybindings handle component-level actions. Escape in vim switches INSERT→NORMAL (does not trigger `chat:cancel`). Most Ctrl+key shortcuts pass through vim to keybindings.
- **UI concept:** Context-grouped table showing all actions with current bindings, inline key capture dialog, chord sequence builder, conflict detection, unbind button, reset-to-default option

### 10. File Suggestion Configuration
- **Settings key:** `fileSuggestion`
- **Description:** Configure custom `@` file autocomplete behavior
- **Details:**
  - `fileSuggestion.type` — `"command"`
  - `fileSuggestion.command` — Path to script (e.g., `"~/.claude/file-suggestion.sh"`)
  - Script receives JSON via stdin: `{"query": "src/comp"}`
  - Script outputs newline-separated file paths (max 15)
- **UI concept:** Script path config with test runner

### 11. Enterprise / Admin Settings Viewer
- **Description:** View and manage managed (IT-enforced) settings
- **Details:**
  - Managed settings locations:
    - macOS: `/Library/Application Support/ClaudeCode/managed-settings.json`
    - Linux/WSL: `/etc/claude-code/managed-settings.json`
    - Windows: `C:\Program Files\ClaudeCode\managed-settings.json`
  - Managed-only keys:
    - `allowManagedHooksOnly` — Only allow managed/SDK hooks
    - `allowManagedPermissionRulesOnly` — Only managed permission rules
    - `disableBypassPermissionsMode` — Prevent permission bypass
    - `allowedMcpServers` / `deniedMcpServers` — MCP allow/denylists
    - `strictKnownMarketplaces` — Enforce marketplace allowlist
  - `companyAnnouncements` — Startup messages (random rotation)
  - `forceLoginMethod` — Restrict to `"claudeai"` or `"console"`
  - `forceLoginOrgUUID` — Auto-select organization
- **UI concept:** Read-only viewer for managed settings, editable for user-scope admin settings

### 12. Session & Cleanup Configuration
- **Settings keys:** `cleanupPeriodDays`, `autoUpdatesChannel`, `teammateMode`, `plansDirectory`
- **Description:** Miscellaneous session and behavior settings
- **Details:**
  - `cleanupPeriodDays` — Delete inactive sessions after N days (default: 30)
  - `autoUpdatesChannel` — `"stable"` or `"latest"` (default)
  - `teammateMode` — Agent teams display: `"auto"`, `"in-process"`, or `"tmux"`
  - `plansDirectory` — Location for plan files (e.g., `"./plans"` or `"~/.claude/plans"`)
- **UI concept:** Simple settings form

### 13. Auth & API Key Helpers
- **Settings keys:** `apiKeyHelper`, `otelHeadersHelper`, `awsAuthRefresh`, `awsCredentialExport`
- **Description:** Configure custom scripts for authentication and credential management
- **Details:**
  - `apiKeyHelper` — Script to generate temporary API keys
  - `otelHeadersHelper` — Script for dynamic OpenTelemetry headers
  - `awsAuthRefresh` — Script to refresh AWS credentials (e.g., `aws sso login --profile myprofile`)
  - `awsCredentialExport` — Script outputting JSON with AWS credentials
- **UI concept:** Script path editors with test/validate buttons

### 14. MCP Approval Settings
- **Settings keys:** `enableAllProjectMcpServers`, `enabledMcpjsonServers`, `disabledMcpjsonServers`
- **Description:** Control which project-level MCP servers are auto-approved
- **Details:**
  - `enableAllProjectMcpServers` — Auto-approve all servers in `.mcp.json`
  - `enabledMcpjsonServers` — Approve specific servers by name
  - `disabledMcpjsonServers` — Reject specific servers by name
- **UI concept:** MCP approval list with per-server toggles

---

## Settings Scope Reference

Claude Code uses a 4-tier scope system (highest to lowest precedence):

| Scope | Location | Shared | Override |
|-------|----------|--------|----------|
| Managed | System-level `managed-settings.json` | IT-deployed | Cannot be overridden |
| User | `~/.claude/settings.json` | No | Lowest precedence |
| Project | `.claude/settings.json` | Yes (git) | Middle precedence |
| Local | `.claude/settings.local.json` | No (gitignored) | High precedence |

Full precedence order:
1. Managed (highest)
2. Command line arguments
3. Local project settings
4. Shared project settings
5. User settings (lowest)

---

## Full Settings JSON Schema

Available at: `https://json.schemastore.org/claude-code-settings.json`

Can be referenced in settings files:
```json
{
  "$schema": "https://json.schemastore.org/claude-code-settings.json"
}
```
