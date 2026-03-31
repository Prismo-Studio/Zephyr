# Contributing to Zephyr

Thanks for your interest in contributing to Zephyr! This guide will help you get started.

## Stack

- **Tauri 2** - Desktop app framework
- **Svelte 5** - Frontend (using runes)
- **Rust** - Backend logic
- **CSS** - Custom properties, no external UI frameworks

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (LTS)
- [pnpm](https://pnpm.io/)

## Setup

```bash
git clone https://github.com/your-fork/Zephyr.git
cd Zephyr
pnpm install
pnpm tauri dev
```

## Branch Naming

Use prefixed branch names based on the type of change:

- `feat/xxx` - New features
- `fix/xxx` - Bug fixes
- `ui/xxx` - UI/UX improvements

## PR Process

1. Fork the repository
2. Create a branch from `dev` using the naming convention above
3. Make your changes
4. Open a PR targeting the `dev` branch

## Code Style

- Use **Svelte 5 runes** (`$state`, `$derived`, `$effect`, etc.)
- Use **CSS custom properties** for theming and shared values
- Do **not** use external UI component frameworks
- Keep components small and focused

<!--
## Recommended Labels

For maintainers, here is a suggested label set:

- bug
- enhancement
- mod-source
- game-support
- ui
- backend
- good first issue
- help wanted
- triage
- priority-high
- priority-low
- wontfix
-->
