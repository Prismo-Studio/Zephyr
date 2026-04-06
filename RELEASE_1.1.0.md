# Zephyr v1.1.0

## New Features

### Gamepad / Controller Support
Full gamepad navigation with D-pad, left stick, and spatial focus system. Virtual on-screen keyboard for text input. Xbox, PlayStation, and Nintendo controllers detected automatically. Gamepad legend bar with context-aware button hints.

### Grid View for Mod Lists
Toggle between list and grid view in both Mods and Browse pages. Grid cards show mod icon, name, author, version, downloads and rating. View preference persisted across sessions.

### Dependencies Tab
New "Dependencies" tab in mod details panel alongside Readme and Changelog. Each dependency shows its Thunderstore icon, name, author, and version. Click any dependency to navigate directly to that mod's details, even if not installed.

### Auto BepInEx Install
Launching a modded game without BepInEx now installs it automatically, waits for installation to complete, then launches the game.

## Improvements

### DPI Scaling
DPI scaling now uses native WebView zoom instead of CSS zoom. Combines zoom factor and DPI scale properly. Sidebar scrollable at high zoom levels.

### Multi-Select Context Menu
Right-click with multiple mods selected shows batch actions with count indicators (Install all, Uninstall all, Enable/Disable).

### Sidebar Persistence
Mod details sidebar stays open after version change, after uninstalling a mod, and refreshes correctly when install completes.

### Copy Content
Copy button now copies clean text instead of raw HTML.

## Bug Fixes

- Version change no longer briefly removes the mod from the list
- "Load more" button hidden when no more mods available
- Scroll resets to top when switching mods or tabs in details panel
- Search filters reset between sessions
- Batch action bar properly positioned with gamepad legend visible
- Clippy warnings fixed

## Testing

- 46 Rust unit tests added
- 16 frontend tests added with Vitest
- `cargo test` and `pnpm test` added to CI workflows

## Contributors

- **Mathis Boulais** (@MathisBls)
- **thefable1**

![Grid View](https://raw.githubusercontent.com/Prismo-Studio/Zephyr/dev/screenshots/showgrid.png)
