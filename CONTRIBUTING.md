# Contributing to Zephyr

First off, thanks for considering contributing to Zephyr. Whether you are reporting a bug, suggesting a feature, improving documentation, or submitting code, your help is appreciated.

This document explains how to get involved and what to expect from the maintainers.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md). Please report unacceptable behavior to [contact@prismo-studios.dev](mailto:contact@prismo-studios.dev).

## Ways to Contribute

There are many ways to help, even without writing code:

- **Reporting bugs** through GitHub Issues
- **Suggesting features** or improvements
- **Improving documentation** (typos, clarifications, translations)
- **Translating Zephyr** into your language (`messages/*.json`)
- **Triaging issues** by reproducing reported bugs and adding context
- **Reviewing pull requests** from other contributors
- **Submitting code** for fixes, features, or refactors

## Reporting Bugs

Before opening a bug report:

1. Check existing issues to avoid duplicates
2. Update to the latest version of Zephyr to confirm the bug still exists
3. Reproduce the issue with a clean profile if possible

When opening a bug report, please use the bug report template and include:

- Zephyr version (visible in Settings)
- Operating system and architecture
- Steps to reproduce
- Expected behavior vs actual behavior
- Logs (if applicable, accessible via `File > Open Zephyr log`)
- Screenshots or screen recordings (if applicable)

For security vulnerabilities, please follow our [Security Policy](SECURITY.md) instead of opening a public issue.

## Suggesting Features

Feature suggestions are welcome. Open an issue with the feature request template and describe:

- The problem you are trying to solve
- The solution you have in mind
- Alternative solutions you have considered
- Any mockups or examples from other tools

Please note that not every feature will be accepted. Zephyr aims to stay focused, fast, and beautiful, so suggestions that significantly expand the scope of the project may be declined or deferred.

## Development Setup

### Prerequisites

- [Node.js](https://nodejs.org/) 18 or later
- [pnpm](https://pnpm.io/) (the project uses pnpm, `npm install -g pnpm` if you don't have it)
- Rust stable toolchain (install via [rustup](https://rustup.rs))
- Tauri prerequisites for your platform: see the [official Tauri docs](https://v2.tauri.app/start/prerequisites/)

### Getting Started

```bash
# Fork the repo on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/Zephyr.git
cd Zephyr

# Add the upstream remote
git remote add upstream https://github.com/Prismo-Studio/Zephyr.git

# Install dependencies
pnpm install

# Run the app in dev mode
pnpm start
```

The `dev` branch is the default working branch. The `master` branch tracks released versions only.

## Branching and Commits

### Branches

- Always branch from `dev`
- Use a descriptive branch name. The following pattern is recommended for external contributors:
  - `feat/short-description` for new features
  - `fix/short-description` for bug fixes
  - `refactor/short-description` for refactoring
  - `docs/short-description` for documentation changes
  - `chore/short-description` for tooling, CI, or maintenance

Maintainers may use their own conventions (e.g. initials-based prefixes).

### Commit Messages

Conventional Commits are encouraged but not strictly required. Keep messages short, in the imperative mood, and explain _why_ in the body if it isn't obvious from the diff.

**Examples:**

```
Add bulk export to zip
Fix updater network timeout on initial check
Refactor mod-list logic into dedicated service
Clarify Linux build requirements in README
```

If you do use a typed prefix (`feat`, `fix`, `refactor`, `docs`, `chore`, `perf`), keep it consistent within your PR. Maintainers may rewrite commit messages on squash-and-merge.

## Code Style

### Svelte and TypeScript

- **Svelte 5 runes only**: use `$state`, `$derived`, `$effect`, `$props`. Do not use `$:`, `export let`, or stores from older Svelte versions
- **TypeScript strict mode** is enabled and must be respected
- **Prettier** formats the code automatically. Run `pnpm lint:fix` (or `pnpm format`) before committing
- **No hardcoded colors or values**: use the design system. CSS custom properties live in `src/app.css`, theme/color tokens are exposed via `src/lib/design-system/tokens.ts`
- **Localization**: any user-facing string must go through Paraglide. Add the key to `messages/en.json` (the base locale) and at least one other locale; other locales can be filled in by translators
- **Component documentation**: each component should have a brief comment at the top describing its purpose, props, and events

### Rust

- **rustfmt** is mandatory. Run `cargo fmt` before committing
- **clippy** must pass without warnings. Run `cargo clippy --all-targets --all-features -- -D warnings`
- **Error handling**: prefer `Result` and `?` consistently. Avoid `unwrap()` and `expect()` in non-test code unless an invariant truly cannot fail
- **Documentation**: public items should have doc comments

### General

- Keep files focused. If a file grows beyond ~500 lines, consider splitting it
- Avoid duplicated logic. Extract shared code into utilities or services
- Comments should explain _why_, not _what_. The code itself should make the _what_ obvious

## Available Scripts

The following scripts are defined in `package.json`:

| Script              | Command                  | What it does                                                    |
| ------------------- | ------------------------ | --------------------------------------------------------------- |
| Start (Tauri)       | `pnpm start`             | Run the desktop app in development mode (alias for `tauri dev`) |
| Dev (web only)      | `pnpm dev`               | Run only the SvelteKit dev server (no Tauri shell)              |
| Build (Tauri)       | `pnpm tauri:build`       | Produce a production installer                                  |
| Format check        | `pnpm lint`              | Check formatting without writing (Prettier `--check`)           |
| Auto-fix formatting | `pnpm lint:fix`          | Run Prettier with `--write --list-different`                    |
| Format              | `pnpm format`            | Format the whole codebase with Prettier `--write`               |
| Type check          | `pnpm check`             | Run `svelte-check` against the TypeScript project               |
| Type check (watch)  | `pnpm check:watch`       | Same as above, re-runs on file changes                          |
| Tests               | `pnpm test`              | Run the Vitest test suite once                                  |
| Tests (watch)       | `pnpm test:watch`        | Re-run tests on file changes                                    |
| Machine translate   | `pnpm machine-translate` | Auto-fill missing translations via Inlang                       |

For Rust:

```bash
# Format
cargo fmt --manifest-path src-tauri/Cargo.toml

# Lint
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

## Pull Requests

### Before Opening a PR

- [ ] Your branch is up to date with `upstream/dev`
- [ ] The app builds (`pnpm tauri:build` succeeds, or at minimum `pnpm start` runs without errors)
- [ ] Format check passes (`pnpm lint`) run `pnpm lint:fix` if it doesn't
- [ ] Type check passes (`pnpm check`)
- [ ] Rust lints pass (`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`)
- [ ] Tests pass (`pnpm test`)
- [ ] You have tested your changes manually
- [ ] You have added or updated relevant documentation

### Opening the PR

1. Push your branch to your fork
2. Open a pull request targeting the `dev` branch of `Prismo-Studio/Zephyr`
3. Fill in the PR template with a clear description of what changed and why
4. Link related issues using `Closes #123` or `Fixes #123`
5. Mark the PR as a draft if it is still a work in progress

### Review Process

- A maintainer will review your PR within a few days
- You may be asked to make changes. Push additional commits to the same branch (no need to force-push for review feedback)
- Once approved, a maintainer will squash and merge your PR into `dev`

### After Merging

Your contribution will be included in the next release. You will be credited in the changelog.

## License

By contributing to Zephyr, you agree that your contributions will be licensed under the same license as the project (see [LICENSE.md](LICENSE.md)).

## Questions

If you have a question that is not answered here, feel free to:

- Open a [Discussion](https://github.com/Prismo-Studio/Zephyr/discussions) on GitHub
- Reach out to the maintainers at [contact@prismo-studios.dev](mailto:contact@prismo-studios.dev)

Thank you for helping make Zephyr better.
