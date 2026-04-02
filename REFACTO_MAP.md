# REFACTO_MAP — Zephyr Codebase Reconnaissance

> Generated 2026-04-02 — Phase 0 RECON complete

---

## CURRENT STATE — FILES OVER 250 LINES (TO SPLIT)

### Frontend (src/)

| File                                                   | Lines | Issue                                                                    |
| ------------------------------------------------------ | ----- | ------------------------------------------------------------------------ |
| `src/routes/+page.svelte`                              | 1,037 | Drag/drop, multi-select, filtering, context menu — ALL in one file       |
| `src/routes/browse/+page.svelte`                       | 565   | Online mod browsing, pagination, filters — duplicates profile page logic |
| `src/routes/prefs/+page.svelte`                        | 536   | Theme, font, lang, cache, directory — all settings in one page           |
| `src/routes/profiles/+page.svelte`                     | 512   | Profile CRUD, sync, icons, import/export                                 |
| `src/routes/config/+page.svelte`                       | 492   | Config editor, value validation                                          |
| `src/routes/dashboard/+page.svelte`                    | 366   | Stats, update checks, dependency viz                                     |
| `src/lib/components/layout/Sidebar.svelte`             | 594   | Game switching, profile switching, launching                             |
| `src/lib/components/dialogs/ShareProfileDialog.svelte` | 456   | Export/import form state                                                 |
| `src/lib/components/mod-list/ModDetails.svelte`        | 423   | Markdown loading, mod actions                                            |
| `src/lib/components/mod-list/ModCard.svelte`           | 406   | Install state, selection, rendering                                      |
| `src/lib/themeSystem.ts`                               | 395   | Color palettes, font system, dynamic CSS                                 |
| `src/lib/components/mod-list/ModListFilters.svelte`    | 354   | Filter state management                                                  |
| `src/lib/types.ts`                                     | 333   | All types in a single file                                               |
| `src/lib/components/mod-list/ToggleModDialog.svelte`   | 247   | Dependency checking                                                      |
| `src/lib/util.ts`                                      | 206   | Mixed utilities                                                          |

### Rust Backend (src-tauri/src/)

| File                                   | Lines | Issue                               |
| -------------------------------------- | ----- | ----------------------------------- |
| `profile/mod.rs`                       | 768   | Core structs + operations mixed     |
| `profile/commands.rs`                  | 632   | 34 commands in one file             |
| `profile/install/queue.rs`             | 628   | Async state machine                 |
| `profile/install/installers/subdir.rs` | 538   | Complex directory detection         |
| `profile/actions.rs`                   | 509   | Core mutation logic                 |
| `db/migrate.rs`                        | 414   | Legacy migration                    |
| `profile/export/modpack.rs`            | 397   | Manifest logic                      |
| `db/mod.rs`                            | 371   | DB init + queries mixed             |
| `config/bepinex/de.rs`                 | 359   | Hand-written parser                 |
| `profile/sync/mod.rs`                  | 350   | WebSocket + API mixed               |
| `thunderstore/ident.rs`                | 348   | String parsing                      |
| `prefs/mod.rs`                         | 339   | Preference state                    |
| `profile/import/mod.rs`                | 323   | Format detection + deser            |
| `config/mod.rs`                        | 318   | Cache + parsing                     |
| `profile/export/changelog.rs`          | 300   | Changelog generation                |
| `profile/install/mod.rs`               | 295   | Install orchestration               |
| `thunderstore/query.rs`                | 290   | Query execution                     |
| `thunderstore/models.rs`               | 290   | API data structures                 |
| `source/thunderstore_adapter.rs`       | 278   | Source adapter                      |
| `thunderstore/mod.rs`                  | 277   | Mod registry                        |
| `profile/install/fs.rs`                | 256   | Profile FS operations               |
| `lib.rs`                               | 254   | App init + 93 command registrations |
| `game/mod_loader.rs`                   | 247   | Mod loader abstraction              |

---

## HARDCODED VALUES

### URLs

- `util.ts`: `https://thunderstore.io/c/${slug}/p/${path}/`
- `util.ts`: `https://raw.githubusercontent.com/Prismo-Studio/Zephyr/refs/heads/dev/images/games/${slug}.webp`
- `util.ts`: `https://gcdn.thunderstore.io/live/repository/icons/${fullName}.png`
- `util.ts`: `https://cdn.discordapp.com/avatars/${discordId}/${avatar}.png`
- `game.svelte.ts`: `https://thunderstore.io/api/experimental/community/${slug}/category/`
- `game/mod.rs`: GitHub API URL for games.json commit check
- `game/mod.rs`: Raw GitHub URL for games.json
- `profile/export/mod.rs`: `https://thunderstore.io/api/experimental/legacyprofile/create/`
- `profile/install/queue.rs`: `https://thunderstore.io/package/download/`

### Magic Numbers

- `toast.ts`: errorDuration=8000, infoDuration=3000, maxCount=5
- `transitions.ts`: 75ms drop-in, 50ms drop-out

---

## DUPLICATED PATTERNS

1. **Multi-select logic**: `+page.svelte` and `browse/+page.svelte` both implement Ctrl/Shift click selection
2. **Mod filtering**: Profile page and browse page have similar QueryModsArgs handling
3. **Dependency dialog**: `RemoveModDialog` and `ToggleModDialog` share identical dependency-checking flow
4. **API error handling**: try-catch + pushToast scattered across components
5. **Icon source resolution**: modIconSrc, gameIconSrc, profileIconSrc — similar patterns

---

## BUSINESS LOGIC IN UI COMPONENTS

| Component                 | API Calls                                        |
| ------------------------- | ------------------------------------------------ |
| Sidebar.svelte            | launchGame, setActive                            |
| ModDetails.svelte         | openModDir, toggleMod, removeMod                 |
| ShareProfileDialog.svelte | export.code, import.readCode, import.profile     |
| InstallModButton.svelte   | install.getDownloadSize, install.mod             |
| RemoveModDialog.svelte    | removeMod, forceRemoveMods, getDependants        |
| ToggleModDialog.svelte    | toggleMod, forceToggleMods, getDependants        |
| LaunchOverlay.svelte      | launch.launchGame, launch.getArgs, setCustomArgs |
| InstallPopover.svelte     | install.allMods, install.clearDownloadCache      |

---

## TAURI COMMANDS — 93 Total

Distributed across 14 `commands.rs` files. Well-organized by domain.

---

## WHAT'S ALREADY GOOD

- API layer (`src/lib/api/`) properly centralizes ALL Tauri invoke calls
- Svelte 5 runes used correctly in state files
- `PersistedState` pattern from 'runed' is clean
- Rust error handling with eyre is consistent
- Database migrations are well-structured
- ModSource trait abstraction allows future multi-source support
- Installer pattern with trait dispatch is clean

---

## TARGET STRUCTURE

See the main refactoring mission document for the full target arborescence.

Key changes from current:

1. `src/lib/types.ts` → split into `src/lib/types/*.types.ts`
2. `src/lib/util.ts` → split into `src/lib/utils/*.ts`
3. `src/lib/themeSystem.ts` → split into services + constants
4. All page routes: extract logic into components + services
5. All large components: split into smaller focused components
6. Rust: profile/mod.rs and profile/commands.rs are the critical splits
