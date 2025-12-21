# Contributing to Claude Code Tool Manager

First off, thank you for considering contributing to Claude Code Tool Manager! It's people like you that make this tool better for everyone.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)
- [Community](#community)

## Code of Conduct

This project and everyone participating in it is expected to uphold a welcoming, inclusive environment. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- [Node.js](https://nodejs.org/) 18 or higher
- [Rust](https://www.rust-lang.org/tools/install) 1.70 or higher
- Platform-specific dependencies for [Tauri](https://tauri.app/start/prerequisites/)
  - **Windows**: Microsoft Visual Studio C++ Build Tools
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`

### Development Setup

1. **Fork the repository**

   Click the "Fork" button on the [GitHub repository](https://github.com/tylergraydev/claude-code-tool-manager).

2. **Clone your fork**
   ```bash
   git clone https://github.com/YOUR_USERNAME/claude-code-tool-manager.git
   cd claude-code-tool-manager
   ```

3. **Install dependencies**
   ```bash
   npm install
   ```

4. **Run in development mode**
   ```bash
   npm run tauri dev
   ```

5. **Run tests**
   ```bash
   npm test
   ```

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check the [existing issues](https://github.com/tylergraydev/claude-code-tool-manager/issues) to see if the problem has already been reported.

When creating a bug report, please use our [bug report template](https://github.com/tylergraydev/claude-code-tool-manager/issues/new?template=bug_report.yml) and include:

- A clear, descriptive title
- Steps to reproduce the issue
- Expected vs actual behavior
- Your operating system and app version
- Any relevant logs or screenshots

### Suggesting Features

Feature requests are welcome! Please use our [feature request template](https://github.com/tylergraydev/claude-code-tool-manager/issues/new?template=feature_request.yml) and include:

- A clear description of the problem you're trying to solve
- Your proposed solution
- Any alternatives you've considered

### Contributing Code

1. **Find an issue to work on**

   Look for issues labeled [`good first issue`](https://github.com/tylergraydev/claude-code-tool-manager/labels/good%20first%20issue) or [`help wanted`](https://github.com/tylergraydev/claude-code-tool-manager/labels/help%20wanted).

2. **Comment on the issue**

   Let others know you're working on it to avoid duplicate effort.

3. **Create a branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

4. **Make your changes**

   Follow the [style guidelines](#style-guidelines) below.

5. **Test your changes**
   ```bash
   npm test
   npm run check
   ```

6. **Commit your changes**

   Write clear, concise commit messages:
   ```bash
   git commit -m "Add feature: description of what you added"
   # or
   git commit -m "Fix: description of what you fixed"
   ```

7. **Push and create a Pull Request**
   ```bash
   git push origin feature/your-feature-name
   ```

## Pull Request Process

1. **Ensure your PR**:
   - Has a clear title and description
   - References any related issues (e.g., "Fixes #123")
   - Includes tests for new functionality
   - Passes all CI checks

2. **PR Review**:
   - A maintainer will review your PR
   - Address any requested changes
   - Once approved, your PR will be merged

3. **After merging**:
   - Delete your branch
   - Your contribution will be included in the next release

## Style Guidelines

### Frontend (Svelte/TypeScript)

- Use TypeScript for all new code
- Follow the existing component structure in `src/lib/components/`
- Use Svelte 5 runes (`$state`, `$derived`, `$effect`)
- Use Tailwind CSS for styling
- Keep components focused and reusable

### Backend (Rust)

- Follow Rust idioms and conventions
- Use `Result` types for error handling
- Document public functions with doc comments
- Keep Tauri commands in `src-tauri/src/commands/`
- Keep business logic in `src-tauri/src/services/`

### General

- Write clear, self-documenting code
- Add comments for complex logic
- Keep functions small and focused
- Use meaningful variable and function names

## Project Structure

```
claude-code-tool-manager/
├── src/                    # SvelteKit frontend
│   ├── lib/
│   │   ├── components/     # Svelte components
│   │   ├── stores/         # Svelte 5 reactive stores
│   │   ├── types/          # TypeScript types
│   │   └── utils/          # Utility functions
│   ├── routes/             # SvelteKit routes
│   └── tests/              # Vitest tests
├── src-tauri/              # Tauri Rust backend
│   └── src/
│       ├── commands/       # Tauri command handlers
│       ├── db/             # Database schema and models
│       └── services/       # Business logic services
└── .github/                # GitHub workflows and templates
```

## Community

- **Questions?** Open a [Discussion](https://github.com/tylergraydev/claude-code-tool-manager/discussions)
- **Found a bug?** Open an [Issue](https://github.com/tylergraydev/claude-code-tool-manager/issues/new?template=bug_report.yml)
- **Have an idea?** Open a [Feature Request](https://github.com/tylergraydev/claude-code-tool-manager/issues/new?template=feature_request.yml)

## Thank You!

Your contributions help make Claude Code Tool Manager better for everyone. Whether it's fixing a typo, reporting a bug, or implementing a new feature - every contribution matters!
