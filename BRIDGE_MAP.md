# Zephyr — Tauri Bridge Map

## Invoke Commands (85+)

### Config (`src/lib/api/config.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `getFiles()` | `get_config_files` | — | `ConfigFile[]` |
| `setEntry(id, value)` | `set_config_entry` | `{file, section, entry, value}` | `void` |
| `resetEntry(id)` | `reset_config_entry` | `{file, section, entry}` | `ConfigValue` |
| `openFile(file)` | `open_config_file` | `{file}` | `void` |
| `deleteFile(file)` | `delete_config_file` | `{file}` | `void` |

### Logger (`src/lib/api/logger.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `openZephyrLog()` | `open_zephyr_log` | — | `void` |
| `logErr(msg)` | `log_err` | `{msg}` | `void` |

### Prefs (`src/lib/api/prefs.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `get()` | `get_prefs` | — | `Prefs` |
| `set(value)` | `set_prefs` | `{value}` | `void` |
| `zoomWindow(value)` | `zoom_window` | `{value}` | `void` |
| `getSystemFonts()` | `get_system_fonts` | — | `string[]` |

### Sources (`src/lib/api/sources.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `getSources()` | `get_sources` | — | `SourceInfo[]` |

### State (`src/lib/api/state.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `isFirstRun()` | `is_first_run` | — | `boolean` |

### Thunderstore (`src/lib/api/thunderstore.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `query(args)` | `query_thunderstore` | `{args: QueryModsArgs}` | `Mod[]` |
| `stopQuerying()` | `stop_querying_thunderstore` | — | `void` |
| `triggerModFetch()` | `trigger_mod_fetch` | — | `void` |
| `getMarkdown(id, type)` | `get_markdown` | `{modRef, kind}` | `string\|null` |
| `setToken(token)` | `set_thunderstore_token` | `{token}` | `void` |
| `hasToken()` | `has_thunderstore_token` | — | `boolean` |
| `clearToken()` | `clear_thunderstore_token` | — | `void` |

### Profile — Main (`src/lib/api/profile/index.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `getGameInfo()` | `get_game_info` | — | `GameInfo` |
| `favoriteGame(slug)` | `favorite_game` | `{slug}` | `void` |
| `setActiveGame(slug)` | `set_active_game` | `{slug}` | `void` |
| `getInfo()` | `get_profile_info` | — | `ManagedGameInfo` |
| `setActive(index)` | `set_active_profile` | `{index}` | `void` |
| `query(args)` | `query_profile` | `{args}` | `ProfileQuery` |
| `isModInstalled(uuid)` | `is_mod_installed` | `{uuid}` | `boolean` |
| `create(name, overridePath)` | `create_profile` | `{name, overridePath}` | `void` |
| `deleteProfile(id)` | `delete_profile` | `{id}` | `void` |
| `rename(name)` | `rename_profile` | `{name}` | `void` |
| `duplicate(name)` | `duplicate_profile` | `{name}` | `void` |
| `removeMod(uuid)` | `remove_mod` | `{uuid}` | `ModActionResponse` |
| `toggleMod(uuid)` | `toggle_mod` | `{uuid}` | `ModActionResponse` |
| `forceRemoveMods(uuids)` | `force_remove_mods` | `{uuids}` | `void` |
| `forceToggleMods(uuids)` | `force_toggle_mods` | `{uuids}` | `void` |
| `setAllModsState(enable)` | `set_all_mods_state` | `{enable}` | `number` |
| `removeDisabledMods()` | `remove_disabled_mods` | — | `number` |
| `getDependants(uuid)` | `get_dependants` | `{uuid}` | `string[]` |
| `openDir()` | `open_profile_dir` | — | `void` |
| `openModDir(uuid)` | `open_mod_dir` | `{uuid}` | `void` |
| `openGameLog()` | `open_game_log` | — | `void` |
| `createDesktopShortcut()` | `create_desktop_shortcut` | — | `void` |
| `getLocalMarkdown(uuid, type)` | `get_local_markdown` | `{uuid, kind}` | `string\|null` |
| `setCustomArgs(args, enabled)` | `set_custom_args` | `{customArgs, enabled}` | `void` |
| `setProfilePath(id, path)` | `set_profile_path` | `{profileId, newPath}` | `void` |
| `forgetProfile(id)` | `forget_profile` | `{profileId}` | `void` |

### Profile — Install (`src/lib/api/profile/install.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `allMods()` | `install_all_mods` | — | `void` |
| `mod(id)` | `install_mod` | `{packageUuid, versionUuid}` | `void` |
| `cancelAll()` | `cancel_all_installs` | — | `void` |
| `clearDownloadCache(soft)` | `clear_download_cache` | `{soft}` | `number` |
| `getDownloadSize(modId)` | `get_download_size` | `{modRef}` | `number` |
| `hasPendingInstallations()` | `has_pending_installations` | — | `boolean` |

### Profile — Update (`src/lib/api/profile/update.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `changeModVersion(id)` | `change_mod_version` | `{packageUuid, versionUuid}` | `void` |
| `mods(uuids, respectIgnored)` | `update_mods` | `{uuids, respectIgnored}` | `void` |
| `ignore(versionUuid)` | `ignore_update` | `{versionUuid}` | `void` |

### Profile — Launch (`src/lib/api/profile/launch.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `launchGame()` | `launch_game` | — | `void` |
| `getArgs()` | `get_launch_args` | — | `string` |
| `openGameDir()` | `open_game_dir` | — | `void` |

### Profile — Import (`src/lib/api/profile/import.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `profile(data, importAll)` | `import_profile` | `{data, importAll}` | `void` |
| `readCode(key)` | `read_profile_code` | `{key}` | `LegacyImportData` |
| `readFile(path)` | `read_profile_file` | `{path}` | `LegacyImportData` |
| `readBase64(base64)` | `read_profile_base64` | `{base64}` | `LegacyImportData` |
| `localMod(path)` | `import_local_mod` | `{path}` | `void` |
| `localModBase64(base64)` | `import_local_mod_base64` | `{base64}` | `void` |
| `getR2modmanInfo(path)` | `get_r2modman_info` | `{path}` | `R2ImportData\|null` |
| `r2modman(path, include)` | `import_r2modman` | `{path, include}` | `void` |

### Profile — Export (`src/lib/api/profile/export.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `code()` | `export_code` | — | `string` |
| `file(dir)` | `export_file` | `{dir}` | `void` |
| `getPackArgs()` | `get_pack_args` | — | `ModpackArgs` |
| `setPackArgs(args)` | `set_pack_args` | `{args}` | `void` |
| `exportPack(dir, args)` | `export_pack` | `{dir, args}` | `void` |
| `uploadPack(args)` | `upload_pack` | `{args}` | `void` |
| `copyDependencyStrings()` | `copy_dependency_strings` | — | `void` |
| `copyDebugInfo()` | `copy_debug_info` | — | `void` |
| `generateChangelog(args, all)` | `generate_changelog` | `{args, all}` | `string` |

### Profile — Sync (`src/lib/api/profile/sync.ts`)
| Function | Command | Params | Return |
|----------|---------|--------|--------|
| `read(id)` | `read_sync_profile` | `{id}` | `SyncImportData` |
| `create()` | `create_sync_profile` | — | `string` |
| `push()` | `push_sync_profile` | — | `void` |
| `clone(id, name)` | `clone_sync_profile` | `{id, name}` | `void` |
| `disconnect(del)` | `disconnect_sync_profile` | `{delete}` | `void` |
| `deleteProfile(id)` | `delete_sync_profile` | `{id}` | `void` |
| `pull()` | `pull_sync_profile` | — | `void` |
| `fetch()` | `fetch_sync_profile` | — | `void` |
| `getOwned()` | `get_owned_sync_profiles` | — | `ListedSyncProfile[]` |
| `login()` | `login` | — | `SyncUser` |
| `logout()` | `logout` | — | `void` |
| `getUser()` | `get_user` | — | `SyncUser\|null` |

---

## Events

### Backend → Frontend (listen)
| Event | Payload | Used In |
|-------|---------|---------|
| `error` | `{name, message}` | `invoke.ts` |
| `profile_changed` | `ProfileInfo` | `+layout.svelte` |
| `game_changed` | `ManagedGameInfo` | `+layout.svelte` |
| `import_profile` | `LegacyImportData\|SyncImportData` | `ImportProfileDialog` |
| `install_mod` | `Mod` | `InstallModDialog` |
| `status_update` | `string\|null` | `Statusbar` |
| `install_event` | `InstallEvent` | `InstallPopover` |
| `mod_query_result` | `Mod[]` | `browse/+page.svelte` |

### Frontend → Backend (emit)
| Event | Payload | Used In |
|-------|---------|---------|
| `reorder_mod` | `{uuid, delta}` | `+page.svelte` |
| `finish_reorder` | `{}` | `+page.svelte` |
