# Zephyr 1.3.0 — Community plugins

This release lands the **plugin system**: a framework for community-built features that ship outside the Zephyr binary, with a shared SDK, dev tooling, and a sandboxed runtime. It also tightens the Zephyr Mods registry pipeline and ships a handful of UX fixes.

![Plugins page](https://raw.githubusercontent.com/Prismo-Studio/Zephyr/master/screenshots/1.3.0/plugin_page.png)

## Plugin system

- **Sandboxed iframe runtime** with a postMessage bridge to Zephyr core. Plugins get sidebar items, their own routes, and access to host capabilities through an explicit API.
- **Per-plugin storage** (`zephyr.storage`, `zephyr.fs`) and notifications (`zephyr.notify`, `zephyr.openExternal`).
- **Event bus** for host signals: `game.launched`, `game.exited`, `game.changed`, `profile.switched`, `locale.changed`, `theme.changed`. Plugins subscribe via `zephyr.on(event, cb)`.
- **Native screen recording** capability (`zephyr.recording.start/stop`) backed by ffmpeg, with auto window detection on Windows (`EnumWindows` + substring match).
- **Theme propagation**: plugins inherit Zephyr's active theme automatically (light, dark, custom, etc.).
- **Dev Mode**: pick a folder on disk, load the plugin live, edit and hot-reload without re-publishing.

![Load plugin](https://raw.githubusercontent.com/Prismo-Studio/Zephyr/master/screenshots/1.3.0/button_load_plugin.png)

## Plugin SDK (`@zephyr-plugin/sdk`)

- Svelte 5 components matching Zephyr: `Toggle`, `Select`, `Button`, `Card`, `Row`, `StatusPill`, `StatCard`.
- Design tokens that track the host's active theme.
- Typed bridge client with autocomplete for events and capabilities.

## Community plugins shipped

- **Captures** (1.2.0): native screen recording, configurable quality (720p to 2160p), framerate (30/60 fps), local mp4 output, preview modal. Auto-record on game launch is wired but disabled in 1.3.0 (coming next).

## Mods & profile improvements

- **Version selector now reflects the installed version** rather than always showing the latest available (Thunderstore + Zephyr Mods).
- **Zephyr Mods installed mods** get a working version dropdown that fetches the version list on-demand from the registry.
- **Browse & Mods pages refresh** automatically after a version change, no manual reload needed.
- Sidebar items for community features show the icon from the plugin manifest (`sidebarIcon`).

## Zephyr Mods registry

- **Validation workflow on every PR**: schema validation, ownership check via full git history, SHA-256 verification of release archives.
- **Manifest schema** locked and published (`schema/registry.schema.json`).
- **Master branch normalised** across all workflows + registry URLs.

## UX

- **Welcome modal** on first launch (7 locales) with a single soft prompt toward the GitHub repo.
- **Modal backdrops** now cover the full window instead of leaving the sidebar visible.
- **Dropdown styling** in plugins matches Zephyr (custom select instead of the native browser one).
- Bumped versions across `package.json`, `tauri.conf.json`, `Cargo.toml`.

## Under the hood

- New Rust modules: `plugins/dev.rs`, `plugins/recording.rs`, `plugins/install_feature` path.
- Filesystem watcher for dev plugins (`notify-debouncer-mini`).
- ffmpeg bundling via `ffmpeg-sidecar` (auto-downloads on first recording).
- SPA fallback enabled on adapter-static so dynamic plugin routes work.

## Documentation

- New [Plugins page](https://docs.zephyr.prismo-studios.dev/guide/plugins) on the docs site.
- New [Plugins wiki page](https://github.com/Prismo-Studio/Zephyr/wiki/Plugins) with screenshots.
- Zephyr-plugin README rewritten with SDK reference and full manifest spec.

---

**Full diff**: [v1.2.14...v1.3.0](https://github.com/Prismo-Studio/Zephyr/compare/v1.2.14...v1.3.0)
