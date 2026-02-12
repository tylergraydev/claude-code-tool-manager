# Suggested Features

A tracking list of suggested features for Claude Code Tool Manager.

## In Progress

### 1. GitHub Token Support
**Status:** In Progress
**Description:** The marketplace uses anonymous GitHub API access (60 req/hr). Adding a personal access token increases this to 5,000 req/hr. Add settings UI for managing tokens and thread them through the GitHub API client.

### 2. Configuration Profiles
**Status:** In Progress
**Description:** Allow users to switch between different sets of globally-enabled MCPs, skills, commands, subagents, and hooks (e.g., "Work" vs "Personal"). Snapshot and restore global configurations with one click.

## Suggested

### 3. Import/Export Configurations
**Description:** Export entire configurations (MCPs, skills, commands, hooks) as shareable JSON/YAML files. Import configurations from files or URLs to quickly set up new environments.

### 4. MCP Health Monitoring
**Description:** Periodically check the health/connectivity of configured MCP servers. Show status indicators (green/yellow/red) and alert users when servers go down.

### 5. Project Templates
**Description:** Save a project's full configuration (MCPs, skills, commands, hooks) as a reusable template. Apply templates to new projects for quick setup.

### 6. Bulk Operations
**Description:** Select multiple items (MCPs, skills, etc.) and perform bulk actions like enable/disable, assign to project, delete, or export.

### 7. Search Across All Types
**Description:** A global search bar that searches across MCPs, skills, commands, subagents, hooks, and projects simultaneously.

### 8. MCP Dependency Management
**Description:** Track dependencies between MCPs (e.g., an MCP that requires another). Warn users when disabling an MCP that others depend on.

### 9. Configuration Diff View
**Description:** Show a diff view when syncing configurations, allowing users to review changes before they're written to disk.

### 10. Scheduled Sync
**Description:** Automatically sync marketplace repositories on a configurable schedule (e.g., daily, weekly) to keep the item catalog fresh.

### 11. Usage Analytics
**Description:** Track which MCPs, skills, and commands are used most frequently. Help users identify underutilized tools and optimize their setup.

### 12. Keyboard Shortcuts
**Description:** Add keyboard shortcuts for common actions like syncing, creating new items, switching between pages, and toggling global items.

### 13. Dark/Light Theme Toggle
**Description:** Add an explicit theme toggle in settings (currently follows system preference). Allow users to force dark or light mode.

### 14. Notification Center
**Description:** A centralized notification center that logs all sync results, import/export operations, and configuration changes with timestamps.
