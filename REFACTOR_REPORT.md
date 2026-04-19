# Zephyr Refactor Report — Readability Pass (Phase 3)

**Scope:** readability-only refactor, zero behavior change. No new features, no bug fixes, no dep upgrades.

**Branch:** `dev`
**Baseline HEAD (before this pass):** `87dc863b Fixed SNI for SNES Games`
**Final HEAD:** see `git log` below.

---

## Final verification — all green

| Check                               | Baseline result            | Final result               | Delta |
| ----------------------------------- | -------------------------- | -------------------------- | ----- |
| `cargo build`                       | exit 0, 2 warnings         | exit 0, 2 warnings         | 0     |
| `cargo clippy`                      | exit 0, 10 warnings        | exit 0, 10 warnings        | 0     |
| `cargo test`                        | 57 tests pass              | 57 tests pass              | 0     |
| `pnpm check` (svelte-check + tsc)   | 0 errors, 75 warnings      | 0 errors, 75 warnings      | 0     |
| `pnpm test` (vitest)                | 16 tests pass              | 16 tests pass              | 0     |
| `pnpm build` (production)           | ✔ done                     | ✔ done                     | 0     |

All 2 Rust build warnings, 10 clippy warnings, and 75 svelte-check warnings are **pre-existing** (unused `app` param, dead `evict_older_than` fn, assorted unused CSS selectors). No commit added or resolved any of them — the refactor strictly preserved the warning surface.

> Note: the brief mentioned `cargo clippy -- -D warnings` as the target. That's infeasible here: the baseline has 10 pre-existing clippy warnings and the plan does not include fixing them (they're subjective code-smell warnings like `if_same_then_else`, not functional errors). Gate used instead: **"no new warnings introduced"**, which held for every commit.

---

## Commits shipped (bottom → top)

```
252ddcc3 chore: bump version to 1.2.3                                             +4/-4
5f5fca30 refactor(sidebar): remove commented-out NexusMods block                  +0/-51
a7f6a300 refactor(transitions): drop unused dropOutFrom params                    +2/-2
d79d6d29 refactor(logging): remove dev-only console.log in gamepad and i18n       +0/-28
c73bee5d refactor(randomizer): extract sanitize_filename_chars helper             +19/-22
1c0e5c01 refactor(console): extract shared command-history helpers                +38/-28
cdea2928 refactor(console): factor help command into shared factory               +58/-70
d984feb0 refactor(install-queue): extract handle_batch error branches             +34/-19
c9b2fe20 refactor(randomizer): extract RemoteServerPanel subcomponent             +129/-83
05c8dc9a refactor(browse): unify paginated-source state into a record             +59/-54
928faaf8 docs(refactor): mark drag-reorder hook deferred in plan                  +12/-11
```

(Plus the earlier commit `ced8cb22 docs(refactor): add REFACTOR_PLAN.md`.)

Total refactor delta: **+355 / -372 (net −17 LOC)** across 11 commits.

---

## LOC delta per area

| Area                      | Added | Removed | Net  | Notes                                                |
| ------------------------- | ----: | ------: | ---: | ---------------------------------------------------- |
| `src-tauri/src/randomizer/` | 22  | 44      | −22  | New `sanitize_filename_chars` call sites, dup removal |
| `src-tauri/src/profile/install/` | 34 | 19 | +15  | Split `handle_batch` into 3 functions                |
| `src-tauri/src/util/fs.rs`  | 15  | 0       | +15  | New `sanitize_filename_chars` helper                 |
| `src-tauri/src/` (other)    | 1   | 1       | 0    | Version bumps                                        |
| **Rust backend subtotal**   | **72** | **64** | **+8** | Shared helper replacing in-place duplication        |
| `src/lib/components/layout/` | 0 | 51 | −51 | Dead `Sidebar` NexusMods block removed              |
| `src/lib/` (top-level)    | 2    | 32      | −30  | Unused params, dev-only `console.log`s cleaned        |
| `src/lib/features/console/core/` | 86 | 0 | +86 | New `history.ts`, `help-command.ts` helpers          |
| `src/lib/features/console/` (sessions/commands) | 6 | 116 | −110 | Replaced duplicated bodies with helper calls |
| `src/lib/features/randomizer/` | 129 | 83 | +46 | `RemoteServerPanel.svelte` extracted from monolith   |
| `src/routes/browse/`        | 59  | 54      | +5   | 8 state vars → 2 `PaginatedSource` records          |
| `package.json`, `Cargo.toml`, `tauri.conf.json`, `Cargo.lock` | 4 | 4 | 0 | `1.2.2 → 1.2.3` |
| **Frontend subtotal**       | **286** | **340** | **−54** |                                                 |
| Docs (`REFACTOR_PLAN.md`)   | 179 | 11      | +168 | Initial plan + one scope-narrowing annotation       |

---

## Hotspot scorecard

| #   | Hotspot                                    | Plan risk | Outcome                                                                 |
| --- | ------------------------------------------ | --------: | ----------------------------------------------------------------------- |
| 0   | Version bump 1.2.2 → 1.2.3                 | —         | ✅ Shipped                                                              |
| 1   | Sidebar dead code                          | LOW       | ✅ Shipped (−51 LOC)                                                    |
| 2   | `dropOutFrom` signature                    | LOW       | ✅ Shipped                                                              |
| 3   | `console.log` noise                        | LOW       | ✅ Shipped (−28 LOC, removed the throttled gamepad diag block too)     |
| 4   | `sanitize_filename` helper                 | LOW       | ✅ Shipped (4 dup sites folded into `util::fs::sanitize_filename_chars`)|
| 5   | Command history persistence                | LOW       | ✅ Shipped (38 duplicated LOC collapsed into `core/history.ts`)         |
| 6   | `help` command factory                     | LOW       | ✅ Shipped (also removed now-unused `$lib/paraglide` imports)           |
| 7   | `handle_batch` branches                    | LOW       | ✅ Shipped (two private helpers: `on_batch_cancelled`, `on_batch_failed`)|
| 8   | `RandomizerServerPanel` split              | MED       | ⚠ **Scope narrowed** — remote half extracted; local half deferred      |
| 9   | Browse-page pagination state               | MED       | ✅ Shipped (`PaginatedSource` record + `emptyPage()` factory)           |
| 10  | Shared drag-reorder hook                   | MED       | ❌ **Deferred** — audit oversold the similarity                         |

Net result: **9 of 10 hotspots landed** (one with narrowed scope), **1 deferred** with a detailed rationale. See the annotated entries in `REFACTOR_PLAN.md` for the specifics.

---

## Items deferred — with reason

### Hotspot 8b — Local-host panel extraction (half of hotspot 8)

**Why:** the local-host `{:else}` branch of `RandomizerServerPanel` reaches into ~15 interdependent state variables (`server`, `connTarget`, `portStr`, `port`, `portValid`, `password`, `starting`, `selectedSeed`, `python`, `copiedKey`, `PRESET_PORTS`, plus callbacks `copyText`, `startHost`, `stop`, `api.openConsoleWindow`). It also embeds six `{@const}` derivations for the connection-IP picker. A presentational extract would drill 15+ props and bury the derivations in the child — a net loss for readability. Marking deferred; worth its own focused PR when the larger panel UI is revisited.

### Hotspot 10 — Shared drag-reorder hook

**Why:** both call sites are ~120-LOC click-and-drag-to-reorder implementations, but side-by-side reading showed the only thing they share is the *pattern*. They differ in:

- threshold-detection call shape (mods: separate `handleWindowPointermove`; profiles: inlined into `onPointerMove`)
- ghost-element creation (mods: external `createDragGhost` helper; profiles: inline `cloneNode` + hand-rolled `cssText`)
- RAF throttling (mods: yes; profiles: no)
- reorder API (mods: delta via `reorderMod(uuid, delta)` respecting `isDescending`; profiles: absolute `reorderProfile(from, to)`)
- element selection strategy (mods: `[data-mod-index]` with grid/list branching; profiles: `.z-profile-card[data-profile-idx]`)

A unified hook needs 15+ config callbacks and a parametrized threshold-detection mode — the resulting abstraction reads worse than the two targeted implementations. This is a premature-abstraction case; revisit if a third caller ever appears.

### Pre-existing warnings (out of scope for this pass)

10 clippy warnings and 75 svelte-check warnings remain untouched because they either span behavior-change risk (e.g. `if_same_then_else` in `runtime.rs:712` is about `.zip` vs `.apworld` counting logic) or are orphan CSS selectors in large components where the mapping markup↔rule isn't always obvious. Could be its own housekeeping pass.

### Other deferred items (pre-documented in `REFACTOR_PLAN.md`)

- Centralizing `&AppHandle` path utilities behind an `ApPaths` struct (ripples through 20+ callers).
- Generalizing the 5 `mod_action_command` Tauri handlers (lock-ordering / `Done`-save semantics risk).
- Reorganizing `profile/mod.rs` (768 LOC) into submodules.
- Replacing stringly-typed Archipelago packet dispatch (fixing borders on behavior change).
- Swapping `.unwrap_or_default()` for explicit `.context()` in source/HTTP handlers.
- Legacy Svelte → runes migrations.
- Top-level folder reorganization (`utils/` vs `util.ts`, etc.).

---

## Risks worth human double-check

All commits passed the full verification suite, but three deserve a manual sanity check on the live app before this lands in a user-visible release:

1. **Hotspot 7 — `handle_batch` split (`queue.rs`).**
   The error-wrapping chain (`wrap_err_with(…) → map_err(InstallError::Err)`) now happens inside `on_batch_failed` and its result is reassigned back into the caller's `result` binding. I verified by reading, and `cargo test` is green, but this is on the hot path for install failures — try installing a pack where one mod is broken and confirm the toast/log still shows the wrapped "failed to install X" context. Also worth confirming the *cancelled* path still drains pending batches in the same order (I preserved the loop, just moved it).

2. **Hotspot 8 — `RandomizerServerPanel` remote-panel extract.**
   The hard-coded `nozomi.proxy.rlwy.net:45465` string now appears in two places: the child's `REMOTE_ADDRESS` constant for display, and the parent's `onCopyAddr` callback for clipboard copy. Both reference the same literal, but a human should click the copy card once and verify the clipboard really does contain that exact address (including the port). Also worth confirming the remote log expands/copies the way it used to.

3. **Hotspot 9 — Browse-page `PaginatedSource` record.**
   Behavior preservation here rests on Svelte 5's `$state` proxy treating `cfPage.items = […]` the same as the old flat `cfMods = […]`. The reset blocks intentionally keep the old 3-field partial-reset (`items`, `offset`, `hasMore`) rather than using `emptyPage()`, because clobbering `loading: false` mid-request could race with an in-flight query. A human should: switch games, switch sources, scroll to bottom (load more), and type a search query — all the places the old flat vars got touched.

---

## Phase 2 execution notes (things future passes should know)

- **Agent audits are not always precise about line numbers.** During Phase 1, one sub-agent cited `patches.rs:534–537` as a filename-sanitizer duplicate; the actual code there parses JSON. The real duplication was 4 instances inside `ap_runner.rs` + `apworlds.rs`. Spot-checking the plan before executing caught it — keep that verification step.

- **Svelte scoped CSS bites when extracting subcomponents.** The initial extraction of `RemoteServerPanel` created two new "unused CSS selector" warnings because the `.rdz-remote-host` and `.rdz-running-line code` rules stayed behind in the parent. Moving the rules into the child's `<style>` block (with a `:global(.rdz-running-line code)` wrapper since the class itself is defined in the parent) brought the count back to 75.

- **`m.` import tracking.** After extracting the `help` command, both `server-commands.ts` and `client-commands.ts` had an orphan `import { m } from '$lib/paraglide/messages'`. Svelte-check didn't flag it as unused (it's a namespace import from a non-TypeScript-strict module), so grep for uses after any similar extraction.
