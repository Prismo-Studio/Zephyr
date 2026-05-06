# Changelog

All notable changes to Zephyr are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/), and this project adheres to
[Semantic Versioning](https://semver.org/).

## [1.2.14] - 2026-05-06

### Added

- Public `CHANGELOG.md` shipped in the Thunderstore package, so the
  Changelog tab of the Zephyr listing now reflects the full version history
  back to v0.7.

### Security

- Bumped `@inlang/paraglide-js` to 2.18 to drop the legacy vulnerable
  `kysely` 0.27 transitive (build-time only).
- Forced patched versions via pnpm overrides:
  - `cookie` → 0.7.2 (out-of-bounds character handling)
  - `esbuild` → 0.25.12 (dev-server request leak)

## [1.2.13] - 2026-05-05

### Added

- Anonymous, opt-in usage telemetry (PostHog EU) with a Privacy toggle in
  Settings. No personal information, no IP, no Discord username sent.
- Page navigation, Discord login/logout, mod install/remove, profile lifecycle,
  and Archipelago events (server start/stop, client connect, seed generation,
  apworld import, runtime install) are now tracked when telemetry is enabled.

### Changed

- Profile names are limited to 25 characters on creation/rename. Long legacy
  names are truncated in the dashboard hero and profile switcher to prevent
  layout overflow.
- Scroll-to-top button moved to bottom-left on Mods and Browse pages so it
  doesn't compete with the version dropdown chevron.
- `project.inlang/.gitignore` is now untracked - Inlang manages it locally
  starting v2.5+, eliminating spurious "file changed" diffs after each build.

### Fixed

- Gamepad d-pad on Standard Mapping controllers (Xbox/DualShock on Windows) no
  longer triggers navigation twice per press, which made selection skip every
  other item.
- Virtual on-screen keyboard d-pad input no longer fires a duplicate event on
  the second frame of a quick tap.
- Pressing Start on the gamepad now opens the Play menu directly instead of
  just focusing the launch button, with the first item focused for d-pad use.
- Install button hover no longer shows a flickery glow shadow; the version
  chevron at its right matches the gradient and stays visible at all states.

### Security

- Updated Rust crypto/runtime crates: `aws-lc-sys` → 0.40, `tar` → 0.4.45,
  `time` → 0.3.47, `bytes` → 1.11, `quinn-proto` → 0.11.14,
  `rustls-webpki` → 0.103.13.
- Updated frontend: `svelte` → 5.55 (XSS during SSR), `vite` → 5.4.21
  (path traversal), `postcss` → 8.5.14 (XSS), `devalue` → 5.8 (proto pollution).
- Forced patched versions via pnpm overrides: `cookie` → 0.7.2,
  `esbuild` → 0.25.12. Bumped `@inlang/paraglide-js` → 2.18 to drop the old
  vulnerable `kysely` 0.27 transitive dependency.

## [1.2.12] - 2026-05-02

### Added

- Randomized and weighted input fields in the Archipelago configurator.

### Changed

- Polished the install button styling and version dropdown chevron animation
  for a less jittery hover experience.

### Fixed

- Plural forms in `timeSince` no longer produce broken strings like "moiss"
  or "mêss" in localized "X months ago" labels.

## [1.2.11] - 2026-04-29

### Added

- Customizable keyboard shortcuts page in Settings, with per-shortcut rebind
  and reset.

### Changed

- Split the largest UI files into smaller, single-purpose components: mod
  list page, browse page, sidebar, profiles page, randomizer panels, and
  preferences sections.
- Centralised hard-coded URLs into `src/lib/constants` and filled in missing
  translations across all seven locales.
- Migrated the toast store to Svelte 5 runes (`$state`) for consistency with
  the rest of the codebase.
- Split `profile/commands.rs` into per-domain submodules (game, profile
  management, mods, dirs, icons) without touching the public command paths.

### Fixed

- Modded launch on Linux when BepInEx isn't yet installed.
- Invisible chevron on the install button's version dropdown when hovering
  while the mod was already installed.

### Removed

- Dead code, unused imports, and 486 orphan i18n keys across all locales
  (about a 44% reduction of the message catalog).

## [1.2.10] - 2026-04-28

### Added

- Toggle switch on each mod card to enable/disable mods directly from the list.
- RPM build target.

### Changed

- Reordered games by favourites, so favourited games appear first in the
  dashboard and selector.
- Replaced the standalone "Vanilla" button with a Play menu offering both
  modded and vanilla launch options.
- Refined the custom-sort drag rail visualization on mod cards (#27).

### Fixed

- ApWorld drag-and-drop installation reliability on Windows.
- Dropdowns no longer stay open after the cursor leaves their bounds.

## [1.2.9] - 2026-04-28

### Added

- Background refresh of ApWorld schemas after the Archipelago runtime
  installs, so newly bundled games appear without a restart.

### Changed

- Orphan ApWorld schemas (no matching `.apworld`) are pruned on runtime
  installation.

### Fixed

- Start inventory validation no longer rejects valid configurations.

## [1.2.8] - 2026-04-26

### Added

- Inline count stepper in the start inventory browser modal for quicker
  quantity adjustments.
- Generic Archipelago schema with progression balancing and accessibility
  options.
- `Ctrl+B` shortcut to toggle the sidebar, plus a "View all shortcuts" modal
  in Settings.
- Scroll-to-top button on the Mods and Browse pages, with smart offset when
  the batch action bar is visible.
- Localized inventory management strings in StartInventoryEditor.
- Ctrl+wheel zoom in the main app, with clarified shortcut labels.
- Browse page item content for several previously empty game communities.

### Changed

- Console output is now selectable and copy-pasteable; start inventory copy
  formatting polished.
- Settings page title aligned with section headings; arrow keys in shortcut
  hints are now quoted for readability.
- Game search treats punctuation as optional, so "R.E.P.O" matches "REPO".

### Fixed

- Console UI polish across the server and client views.

## [1.2.7] - 2026-04-25

### Added

- Highlight on changed top-level YAML keys in the Archipelago config editor.

### Changed

- Replaced the bundled Archipelago runtime with a unified install flow.
- Refreshed the Multiplayer panel design.
- Default DPI raised to 110% for better legibility.
- `query_profile` now runs on the blocking thread pool to avoid stalling
  the main async runtime.
- `RandomizerServerPanel` split into Python, Players, Seeds, and Host
  sub-sections for maintainability.
- Process command extension moved to a shared util module.

### Fixed

- Windows freezes during Archipelago seed generation by hiding the
  subprocess console windows.
- Randomizer UI no longer freezes mid-install.

## [1.2.6] - 2026-04-24

### Added

- View mode options (grid/list) in the mod list filters with responsive layout
  adjustments.
- Unknown mods management: detect mods present in the profile but missing
  from the active source, with a one-click removal flow.
- Start inventory editor for Archipelago.
- Setup guide opens in the system browser.
- Process permissions in the main Tauri capability config.

### Changed

- Renamed APWorld metadata handling features to "Archipelago" branding.

### Fixed

- AppImage open-folder action.
- Custom folder selection for multi-ApWorld setups.
- Duplicate "Your Games" title and section icons cleaned up on the dashboard.

## [1.2.5] - 2026-04-22

### Changed

- Replaced Railway remote hosting with archipelago.gg as the default remote
  Archipelago server.

## [1.2.4] - 2026-04-19

### Added

- Page-size selector in the mod list with localised labels.
- Custom background media feature: upload images/videos as app background.
- Quick Actions menu and retractable server pane in the Randomizer page.

### Changed

- Mod search now supports alphanumeric compact matching (ignores punctuation
  and spacing differences).

### Fixed

- macOS fullscreen behaviour.
- Pointer cursor on README/changelog links inside mod details.

## [1.2.3] - 2026-04-19

### Added

- F11 fullscreen toggle.
- Loading data messages for the randomizer in all supported locales.
- Bundled Python 3.13.9 fallback for systems without a system Python.

### Fixed

- AppImage Python integration.
- Randomizer host configuration patch.

## [1.2.2] - 2026-04-18

### Added

- Custom Archipelago clients and patches support.
- ApWorld download and runtime install through the in-app catalog.
- Custom theme picker with a ColorPicker component.
- On-disk cache for remote mod icons.
- Zephyr Console (server + client views, dual-process).
- SNES game ROM integration via SNI.

### Changed

- Refactored Randomizer page layout with a resizable right pane.
- Cleaned up server panel visuals.
- Tooltip styles wrap properly on long content.
- Shared launch-with-BepInEx fallback logic between sidebar and dashboard.

### Fixed

- SNI patch handling messages and user instructions clarified.
- Tracker `items_handling` set to 0 so the real game client receives items.
- Removed `target="_blank"` on guide URLs that opened blank tabs.

## [1.2.1] - 2026-04-17

### Added

- Global Search component (`Ctrl+F`) with category-tagged results.

### Fixed

- Deep link handling on macOS via Apple Events.
- Auth callback channel capacity increased to avoid dropped login redirects.

## [1.2.0] - 2026-04-17

### Added

- Cloud sync UI with auto-push, fork import, unsync, and restore dialogs.
- Cross-game profile restore dialog when no profiles match the active game.
- Ko-fi sponsor link in the README and About section.
- Dashboard redesign with favourite hero tiles and grouped games.
- Randomizer translations across all 7 supported languages.
- Catalog layout and Archipelago subtitle on the randomizer page.

### Changed

- Sync now points at `api.zephyr.prismo-studios.dev`; pull-before-launch is a
  toggle in Settings.
- Profile rename is pushed to cloud so the manifest name updates server-side.
- Smooth gradient hover on primary/vanilla buttons; toast styling in light
  mode.
- Replaced all hard-coded cyan colours with theme variables.
- Replaced remaining hard-coded strings with i18n keys.
- Auto-regenerate seed on player change and removed the redundant "Stop
  remote" button.
- Improved DPI scaling logic and event handling.

### Fixed

- Setup links in Archipelago.
- Restore dialogs now close after their action via `bind:open`.
- Cross-game restore dialog only opens on manual click, not on mount.

## [1.1.3] - 2026-04-07

Maintenance release; superseded shortly by 1.1.2.

## [1.1.2] - 2026-04-07

### Changed

- Drag-and-drop adjustment on the mod list.
- Reorder profile interaction polish.

## [1.1.1] - 2026-04-07

### Added

- Arabic locale.

### Fixed

- Quick actions reactivity on the randomizer page.

## [1.1.0] - 2026-04-06

### Added

- Ko-fi support badge and sponsor link.

### Changed

- Updated launch icon.

## [1.0.2] - 2026-04-06

### Added

- Gamepad navigation across all main pages.
- Virtual on-screen keyboard component for text input via gamepad.
- Grid and list view toggle for mod lists.
- Dependencies tab in the mod details panel.
- Multi-select context menu for batch actions.
- Auto-install BepInEx on first modded launch when missing.
- Unit tests with CI integration.

### Changed

- Extracted shared utilities and centralised constants; split
  `ModDependencyList` into smaller components.
- DPI scaling fully fixed on Windows.
- Updater artefacts enabled in CI.

### Security

- API keys obfuscated in the bundled binary.

## [1.0.1] - 2026-04-05

### Added

- Russian language.
- Per-game configuration UI.
- Auto-update flow.
- `NumberInput` component.
- Clickable category tags (Ctrl+click).
- Community source adapter.

### Fixed

- Version change no longer deletes the mod or hides the load-more button.
- Multiple-tag selection on browse.

## [1.0.0] - 2026-04-05

First stable release of the Zephyr fork. Per-game config and auto-update
infrastructure stabilised, packaging cleaned up, internal `cargo check`
errors resolved.

## [0.9.0] - 2026-04-04

### Added

- Full CurseForge integration: pagination, changelogs, source badges,
  selection and filter handling.
- Clickable category tags with teal hover and search reset on app start.

### Fixed

- CurseForge filter only shown on supported games.
- Tag selection edge cases.

## [0.8.0] - 2026-04-04

### Added

- Multi-select mod detail navigation on Mods and Browse pages.
- Vanilla launch button.

### Changed

- Tailwind CSS bumped to v4.2.2.
- Deferred profile-code reads until Thunderstore fetch completes.

### Fixed

- macOS warning on launch.
- Settings version footer alignment and `PathBuf` handling.
- Missing `PathExt` import in `platform.rs`.

### Removed

- NexusMods integration is temporarily commented out pending a rework.
- CurseForge toggle visually marked "coming soon" via tooltip.

## [0.7.0] - 2026-04-03

Initial public release of the Zephyr fork.

### Added

- Multi-source integration: NexusMods and CurseForge adapters alongside
  Thunderstore, with a source switcher in Browse (4500+ NexusMods games).
- AppImage build scripts and Linux packaging.
- Cleaned project root layout from the upstream fork.
