# Roadmap

Feature gap analysis between the Tool Manager and Claude Code's current capabilities. Organized by priority.

---

## High Priority

Core features users will expect to configure through the Tool Manager.

### ~~Hooks: Missing Event Types~~ (Done)

All 14 missing hook events added — types, metadata, store ordering, templates, sound presets, and tests.

- [x] `InstructionsLoaded` — fires when CLAUDE.md or rules are loaded (matcher: load reason)
- [x] `PostToolUseFailure` — fires after a tool fails (separate from PostToolUse success path)
- [x] `SubagentStart` — fires when a subagent spawns (matcher: agent type name)
- [x] `TaskCompleted` — fires when a background task completes
- [x] `TeammateIdle` — fires before a teammate idles (agent teams)
- [x] `StopFailure` — fires on error (matcher: `rate_limit`, `authentication_failed`, `server_error`)
- [x] `ConfigChange` — fires when settings change (matcher: `user_settings`, `project_settings`, `policy_settings`)
- [x] `CwdChanged` — fires when the working directory changes
- [x] `FileChanged` — fires on file changes (matcher: filename)
- [x] `WorktreeCreate` — fires when a git worktree is created
- [x] `WorktreeRemove` — fires when a git worktree is removed
- [x] `PostCompact` — fires after context compaction
- [x] `Elicitation` — fires on MCP user input request
- [x] `ElicitationResult` — fires on MCP elicitation response

### ~~Hooks: Missing Hook Types~~ (Done)

Added `http` and `agent` hook types with full DB, backend, and UI support.

- [x] `http` — POST to a URL with headers and env var substitution (`$VARIABLE`)
  - Fields: `url`, `headers`, `allowedEnvVars`
- [x] `agent` — spawn a subagent to process the hook event

### ~~Hooks: Missing Hook Fields~~ (Done)

Added 5 universal hook fields with DB columns, settings.json serialization, and form UI.

- [x] `if` — permission rule syntax filter (e.g., `Bash(rm *)`)
- [x] `statusMessage` — custom spinner text while hook runs
- [x] `once` — run only once per session
- [x] `async` — run in background (command hooks)
- [x] `shell` — `bash` (default) or `powershell`

### ~~Permission Modes~~ (Done)

The app exposes `default`, `allowEdits`, `bypassPermissions`. Added:

- [x] `plan` — read-only exploration mode
- [x] `auto` — background safety checks (research preview)
- [x] `dontAsk` — auto-deny unless pre-approved

### ~~Effort Level Configuration~~ (Done)

Added `effortLevel` scoped setting with full DB, backend, settings writer, and UI support. Also added effort to skill frontmatter.

- [x] Add `effortLevel` setting (`low`, `medium`, `high`, `max`)
- [x] Surface in model/settings UI
- [x] Support effort in subagent frontmatter
- [x] Support effort in skill frontmatter
- [x] Env var toggle: `CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING`

### ~~Subagent Frontmatter: New Fields~~ (Done)

Added 9 missing frontmatter fields with full DB, backend, writer, scanner, and UI support.

- [x] `disallowedTools` — tool denylist (complement of existing `tools` allowlist)
- [x] `maxTurns` — limit iterations
- [x] `memory` — persistent memory scope (`user`, `project`, `local`)
- [x] `background: true` — always run in background
- [x] `effort` — effort level override
- [x] `isolation: worktree` — run in a git worktree
- [x] `hooks` — scoped hooks (run only within this subagent)
- [x] `mcpServers` — scoped MCP servers (available only to this subagent)
- [x] `initialPrompt` — auto-submitted first prompt

### ~~Skill Frontmatter: New Fields~~ (Done)

Added 6 missing frontmatter fields with full DB, backend, writer, scanner, and UI support.

- [x] `context: fork` — run in subagent context
- [x] `agent` — which subagent type to use
- [x] `hooks` — lifecycle hooks scoped to this skill
- [x] `paths` — glob patterns for auto-loading (comma-separated or list)
- [x] `shell` — `bash` or `powershell`
- [x] `once` — run only once per session

### ~~Rules Directory Management~~ (Done)

Added full rules management with DB, backend, filesystem scanning, and UI.

- [x] Manage `.claude/rules/` (project) and `~/.claude/rules/` (user) directories
- [x] Create/edit/delete rule files (markdown with frontmatter)
- [x] Support `paths` frontmatter for glob-based conditional loading
- [x] Show which rules are active for a given file context
- [x] Symlink support for shared rules across projects

### ~~WebSocket MCP Transport~~ (Done)

Added `ws` as a 4th MCP transport type with full DB, config writer, and UI support.

- [x] Add `ws` transport type for MCP servers (`wss://` URLs)
- [x] UI support in MCP creation/edit forms

---

## Medium Priority

Power user and advanced configuration features.

### Auto Mode Configuration

New settings block for the `auto` permission mode:

- [ ] `autoMode.environment` — trusted infrastructure descriptions (prose)
- [ ] `autoMode.allow` — actions to allow (prose descriptions)
- [ ] `autoMode.soft_deny` — actions to block (prose descriptions)

### Model Overrides & Extended Context

- [ ] `modelOverrides` setting — map Anthropic model IDs to provider-specific IDs (Bedrock, Vertex, Foundry)
- [ ] Extended context suffixes: `sonnet[1m]`, `opus[1m]`
- [ ] `opusplan` alias (Opus in plan mode, Sonnet otherwise)
- [ ] `availableModels` setting — restrict which models users can select
- [x] Custom model env vars (`ANTHROPIC_CUSTOM_MODEL_OPTION`, `_NAME`, `_DESCRIPTION`) — added to Known Env Vars picker
- [x] Third-party provider pinning env vars (`ANTHROPIC_DEFAULT_OPUS_MODEL`, etc.) — added to Known Env Vars picker

### Expanded Sandbox Settings

New sandbox knobs not yet in the app:

- [ ] `sandbox.filesystem.allowRead` — re-allow reading within denied regions
- [ ] `sandbox.filesystem.denyRead` — paths subprocesses cannot read
- [ ] `sandbox.filesystem.allowUnixSockets` — allow specific Unix sockets (e.g., Docker)
- [ ] `sandbox.network.httpProxyPort` — custom HTTP proxy port
- [ ] `sandbox.network.socksProxyPort` — custom SOCKS proxy port
- [ ] `sandbox.network.allowManagedDomainsOnly` — only allow managed domain lists
- [ ] `sandbox.enableWeakerNestedSandbox` — weaker mode for Docker compatibility
- [ ] `sandbox.allowUnsandboxedCommands` — allow `dangerouslyDisableSandbox` parameter

### Permission Rule Syntax Expansion

The permission rule builder needs new matcher types:

- [ ] `Agent(subagent-name)` — match specific subagent spawning
- [ ] `Skill(skill-name)` / `Skill(skill *)` — match skill invocation
- [ ] `WebFetch(domain:example.com)` — domain-level filtering for web fetches

### ~~Agent Memory~~ (Done)

Added per-subagent persistent memory system with full backend service, Tauri commands, store, and UI.

- [x] Manage `.claude/agent-memory/` (project) and `~/.claude/agent-memory/` (user) directories
- [x] Per-agent `MEMORY.md` index files
- [x] Browse/edit agent memory from the subagent detail view

### ~~New Settings Keys~~ (Done)

Added 6 new settings keys with full DB, backend, settings writer, and UI support.

- [x] `autoMemoryEnabled` / `autoMemoryDirectory` — auto memory toggle and custom path
- [x] `claudeMdExcludes` — glob patterns to skip CLAUDE.md files (monorepo support)
- [x] `agent` — default subagent for a project
- [x] `attribution.enabled` / `attribution.rules` — contribution tracking with file patterns
- [x] `disableBypassPermissionsMode` — already existed in managed settings
- [x] `disableAutoMode` — prevent auto mode

---

## Lower Priority

Enterprise, niche, or session-scoped features.

### Agent Teams

- [ ] `agentTeamEnabled` setting
- [ ] UI to visualize/configure team composition
- [ ] Team size, model per agent, display mode settings

### Managed Settings (Enterprise)

Admin-deployed, read-only configuration:

- [ ] `allowManagedPermissionRulesOnly` — only enforce managed permission rules
- [ ] `allowManagedHooksOnly` — only load managed hooks
- [ ] `allowManagedMcpServersOnly` — only use managed MCP servers
- [ ] `forceLoginMethod` / `forceLoginOrgUUID` — lock authentication
- [ ] `allowedChannelPlugins` — restrict channel plugins
- [ ] `sandbox.filesystem.allowManagedReadPathsOnly` — restrict to managed read paths
- [ ] Better read-only display for managed settings in the Admin tab

### Cron / Scheduled Tasks

Built-in scheduling (mostly session-scoped, but desktop/cloud have persistent variants):

- [ ] Expose `/loop` interval configuration
- [ ] Expose `/schedule` for cloud/desktop persistent scheduling
- [ ] CronCreate/CronList/CronDelete integration
- [ ] Visual cron expression builder

### Plugin Marketplace: npm Source

- [ ] Add `npm` as a marketplace source type (`@scope/package`)
- [ ] npm package resolution and installation flow

### CLI Startup Flags

Surface key CLI flags as launchable presets or project config:

- [ ] `--agent` — run session as specific subagent
- [ ] `--baremode` — minimal plugin startup
- [ ] `--system-prompt` / `--append-system-prompt` — custom system prompts
- [ ] `--permissions` / `--allowedTools` / `--disallowedTools` — runtime overrides

---

## Tracking

Last audited: 2026-03-29
Audited against: Claude Code capabilities as of March 2026
