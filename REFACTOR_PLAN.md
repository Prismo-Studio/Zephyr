# Zephyr Refactor Plan — Readability Pass (Phase 1 Audit)

**Scope:** readability-only refactor, **zero behavior change**. No new features, no bug fixes, no dep changes, no folder reorg, no Svelte 4→5 runes migrations.

**Status:** Phase 1 approved by user; Phase 2 execution in progress.

---

## Structure summary

| Area              | Location             | LOC     | Language               |
| ----------------- | -------------------- | ------- | ---------------------- |
| Rust backend      | `src-tauri/src/`     | ~21,400 | Rust 2021 (edition)    |
| Frontend app      | `src/`               | ~30,100 | Svelte 5 (runes) + TS  |
| Shared UI         | `src/lib/components` | ~7,700  | Svelte                 |
| Shared feature    | `src/lib/features`   | ~7,800  | Svelte + `.svelte.ts`  |
| Routes            | `src/routes`         | ~7,900  | Svelte `+page.svelte`  |
| Tests             | `src/lib/utils/*.test.ts`, `src-tauri/src/{config/bepinex,randomizer}/tests.rs` | ~400 | Vitest + Rust `#[test]` |

**Toolchain commands** (confirm each passes after every commit):
- `cd src-tauri && cargo build`
- `cd src-tauri && cargo clippy -- -D warnings`
- `pnpm check` (runs `svelte-kit sync && svelte-check`)
- `pnpm build`
- `pnpm test` (vitest) + `cd src-tauri && cargo test`

Key module boundaries to respect:
- Tauri `#[command]` signatures in `src-tauri/src/**/commands.rs` → frozen
- `pub` items that cross `mod` boundaries → frozen
- Exported `.svelte` component props → frozen
- `$lib/api/*` function signatures (used by many routes) → frozen

---

## Top 10 hotspots — ordered **lowest risk first**

### 1. Dead commented-out NexusMods block in `Sidebar.svelte` (LOW)

- **File:** [Sidebar.svelte:14–30](src/lib/components/layout/Sidebar.svelte:14)
- **Diagnosis:** 17 lines of commented-out imports, `$state` declarations, and an `$effect` referencing a future NexusMods source. Not a disabled-but-ready switch — just crufty dead code blocking the top of the script.
- **Proposed change:** Delete the block. The git history preserves it if anyone needs it later.
- **Risk:** low — pure deletion, no references anywhere.

### 2. Unused destructure in `dropOutFrom` (LOW)

- **File:** [transitions.ts:14](src/lib/transitions.ts:14)
- **Diagnosis:** `export function dropOutFrom({}: { x?: number; y?: number })` destructures nothing and ignores both params; `dropOut = dropOutFrom({ y: -5 })` at line 8 passes a value that does nothing.
- **Proposed change:** Drop the parameter entirely and the call-site argument.
- **Risk:** low — only one caller in the module.

### 3. Trivial `console.log` noise (LOW)

- **Files:** [gamepad.svelte.ts:786](src/lib/gamepad.svelte.ts:786), [i18nCore.svelte.ts:15](src/lib/i18nCore.svelte.ts:15)
- **Diagnosis:** `console.log` calls shipped in production (gamepad telemetry, i18n locale updates).
- **Proposed change:** Delete them. No replacement.
- **Risk:** low — output-only.

### 4. Extract `sanitize_filename` helper in randomizer (LOW)

- **Files:** [ap_runner.rs:279–283, 367–370, 535–537](src-tauri/src/randomizer/ap_runner.rs:279), [apworlds.rs:268–272](src-tauri/src/randomizer/apworlds.rs:268)
- **Diagnosis:** The same `c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' }` char-by-char sanitizer appears 3× inside `ap_runner.rs` (one variant also allows `.`) and once in `apworlds.rs` with a wider allow-list.
- **Proposed change:** Add `fn sanitize_filename(name: &str, extra_allowed: &[char]) -> String` in `src-tauri/src/util/fs.rs`. Replace the four inline loops. Preserve the `"player"` fallback in `ap_runner.rs:283` at the call site.
- **Risk:** low — function-private callers only.

### 5. Consolidate `loadHistory`/`pushHistory` duplication (LOW)

- **Files:** [server-session.svelte.ts:150–168](src/lib/features/console/server/server-session.svelte.ts:150), [client-session.svelte.ts:418–436](src/lib/features/console/client/client-session.svelte.ts:418)
- **Diagnosis:** Word-for-word identical `loadHistory()` / `pushHistory()` pair across two session classes — 38 LOC duplicated.
- **Proposed change:** Add helper in `src/lib/features/console/core/history.ts`. Call from both sessions.
- **Risk:** low — private methods only.

### 6. Unify `help` command implementation (LOW)

- **Files:** [server-commands.ts:26–50](src/lib/features/console/server/server-commands.ts:26), [client-commands.ts:20–44](src/lib/features/console/client/client-commands.ts:20)
- **Diagnosis:** Two near-identical `help` command implementations differing only in the prefix character (`/` vs `!`).
- **Proposed change:** Add `createHelpCommand(prefix)` factory in `src/lib/features/console/core/help-command.ts`.
- **Risk:** low — factory only abstracts the prefix.

### 7. Extract `handle_batch` error branches in install queue (LOW)

- **File:** [queue.rs:304–343](src-tauri/src/profile/install/queue.rs:304)
- **Diagnosis:** 40-line async function with inline 3-arm match (Ok / Cancelled / Err). Control flow mixes happy path, rollback, and queue-fanout side effects.
- **Proposed change:** Split the two error arms into `handle_batch_cancelled` and `handle_batch_failed` private helpers. Keep `handle_batch` as outer dispatch.
- **Risk:** low — all private to the module.

### 8. Split `RandomizerServerPanel.svelte` (MED)

- **File:** [RandomizerServerPanel.svelte](src/lib/features/randomizer/RandomizerServerPanel.svelte) (1698 LOC)
- **Diagnosis:** Mixes remote-server WS upload, local-host Python form, and seed dialogs. 30+ `$state` vars, nested `{#if hostMode === 'remote'}` markup.
- **Proposed change:** Extract `RemoteServerPanel.svelte` and `LocalHostPanel.svelte` children. Parent keeps shared state (selected seed, logs, dialogs). Stop after host-mode split and reassess.
- **Risk:** med — no exported prop changes, but three files instead of one.
- **Alternative noted:** Full 5-way split — rejected as too ambitious for one pass.
- **Executed scope (narrower than proposed):** Only the remote branch was extracted into `RemoteServerPanel.svelte`. The local-host branch has ~15 interdependent props (`server`, `connTarget`, `portStr`, `port`, `portValid`, `password`, `starting`, `PRESET_PORTS`, `python`, `selectedSeed`, `copiedKey`, `copyText`, `startHost`, `stop`, `api.openConsoleWindow`) plus inline `@const` derivations for the connection picker, which made the prop surface bigger than the readability win justified in one pass. **Deferred** as a dedicated follow-up.

### 9. Unify browse-page pagination state (MED)

- **File:** [browse/+page.svelte:143–340](src/routes/browse/+page.svelte:143)
- **Diagnosis:** Nine flat `$state` vars track identical shapes across three sources (Thunderstore / CurseForge / Zephyr Server). `refresh()` and `loadMore*()` branch on source identity to do structurally identical work.
- **Proposed change:** Introduce `PaginatedSource` record and collapse `loadMore*` into a generic. Keep data flow identical.
- **Risk:** med — touches primary browse-page state; cross-check manually.

### 10. Extract drag-reorder hook shared by profile + mod pages (MED)

- **Files:** [+page.svelte:62–170](src/routes/+page.svelte:62), [profiles/+page.svelte:318–442](src/routes/profiles/+page.svelte:318)
- **Diagnosis:** Two ~120 LOC drag-and-reorder implementations duplicate the full pointer-capture + threshold + ghost-element cloning + RAF-scheduling dance.
- **Proposed change:** Hoist to `src/lib/utils/drag-reorder.svelte.ts` factory returning pointer handlers.
- **Risk:** med — shared hook is new surface for these two callers only.
- **Executed:** **Deferred.** Side-by-side reading showed the audit oversold the similarity. The two implementations differ in: (a) threshold-detection call structure (mods: separate `handleWindowPointermove`; profiles: inline in `onPointerMove`), (b) ghost-element creation (mods: external `createDragGhost` helper; profiles: inline `cloneNode` + hand-rolled cssText), (c) RAF throttling (mods: yes; profiles: no), (d) reorder API (mods: delta via `reorderMod(uuid, delta)` with ascending/descending direction; profiles: absolute `reorderProfile(from, to)`), (e) element selection strategy (mods: `[data-mod-index]` + grid/list detection; profiles: `.z-profile-card[data-profile-idx]`). A unified hook would need 15+ config callbacks and a parametrized threshold-detection mode — the resulting abstraction would read worse than the two targeted implementations. This is the premature-abstraction case; deferred as a dedicated follow-up if a third caller ever appears.

---

## Decisions explicitly deferred (out of scope for this pass)

| Item | Why deferred |
| ---- | ------------ |
| Centralizing `&AppHandle` path utilities behind an `ApPaths` struct | Med–high risk, ripples through 20+ randomizer callers. |
| Generalizing the 5 `mod_action_command` Tauri handlers | Medium risk of breaking lock ordering / `Done` save semantics. |
| Reorganizing `profile/mod.rs` (768 LOC) into submodules | Either cosmetic (low value) or real split (broken `pub use` risk). |
| Replacing stringly-typed Archipelago packet dispatch | Cast-assertions hide real type holes; fixing borders on behavior change. |
| Swapping `.unwrap_or_default()` for explicit `.context()` in source/HTTP handlers | Error-path semantics — bug-fix disguised as refactoring. |
| Converting remaining legacy Svelte components to runes | Explicitly out of scope. |
| Top-level folder reorganization (`utils/` vs `util.ts`, `types/` vs `types.ts`) | Explicitly out of scope. |

---

## Verification gate per commit

After each hotspot commit:

```sh
cd src-tauri && cargo build && cargo clippy -- -D warnings && cargo test
cd - && pnpm check && pnpm build && pnpm test
```

If anything turns red: revert, diagnose, retry. Never ship red.

---

## Commit plan

| # | Commit | Hotspot |
| - | ------ | ------- |
| 0 | `chore: bump version to 1.2.3` | (version bump, separate from refactors) |
| 1 | `refactor(sidebar): remove commented-out nexusmods block` | #1 |
| 2 | `refactor(transitions): drop unused dropOutFrom params` | #2 |
| 3 | `refactor(logging): remove stray console.log in gamepad/i18n` | #3 |
| 4 | `refactor(randomizer): extract sanitize_filename helper` | #4 |
| 5 | `refactor(console): consolidate history persistence` | #5 |
| 6 | `refactor(console): factor help command out of server+client` | #6 |
| 7 | `refactor(install-queue): extract handle_batch error branches` | #7 |
| 8 | `refactor(randomizer): split RandomizerServerPanel into host-mode subcomponents` | #8 |
| 9 | `refactor(browse): unify paginated-source state` | #9 |
| 10 | `refactor(drag-reorder): extract shared svelte hook` | #10 |

---

**Progress log** (filled in during Phase 2):

- [x] 0. Version bump 1.2.2 → 1.2.3
- [x] 1. Sidebar dead code
- [x] 2. dropOutFrom signature
- [x] 3. console.log noise
- [x] 4. sanitize_filename helper
- [x] 5. history persistence
- [x] 6. help command factory
- [x] 7. handle_batch split
- [x] 8. RandomizerServerPanel split (remote half only; local half deferred)
- [x] 9. browse pagination
- [ ] 10. drag-reorder hook — **deferred, see hotspot note above**
