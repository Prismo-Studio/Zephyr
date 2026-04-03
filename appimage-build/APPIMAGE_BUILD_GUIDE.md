# AppImage Distribution Fix Guide

## Overview

This folder contains all the tools needed to build and distribute AppImages for Zephyr on Linux.

**Location:** `appimage-build/` folder (organized for clean project structure)

## Problem Solved ✅

When building the AppImage locally, Tauri creates symlinks in the `.AppDir` directory that point to absolute paths. These break when distributed to other systems.

**Solution Applied:**

- `.DirIcon` → `Zephyr.png` ✅
- `Zephyr.desktop` → `usr/share/applications/Zephyr.desktop` ✅

## Quick Start

Choose one of these approaches:

### Option 1: Complete Workflow (Recommended)

```bash
pnpm tauri:build:complete
```

Or directly:

```bash
./appimage-build/build_and_fix.sh
```

- Runs full Tauri build
- Automatically fixes symlinks
- Attempts to generate AppImage
- Reports results

### Option 2: Manual AppImage Generation (Fallback)

If `pnpm tauri build` has issues with linuxdeploy:

```bash
pnpm tauri build                       # Creates AppDir beforehand
pnpm tauri:appimage:generate           # Manually generates AppImage with plugin
```

Or directly:

```bash
pnpm tauri build
./appimage-build/generate_appimage.sh
```

### Option 3: Just Fix Existing AppDir

```bash
pnpm tauri:appimage:fix
```

Or directly:

```bash
./appimage-build/post_build_fix.sh
```

- Fixes symlinks in existing AppDir
- Optionally generates AppImage if plugin is available

## Using npm Scripts

Add these to your workflow:

```bash
# Build and fix everything
pnpm tauri:build:complete

# Manual AppImage generation
pnpm tauri:appimage:generate

# Just fix symlinks
pnpm tauri:appimage:fix
```

## What Each Script Does

All scripts are located in the `appimage-build/` folder:

| Script                                    | Purpose                                                     |
| ----------------------------------------- | ----------------------------------------------------------- |
| `appimage-build/build_and_fix.sh`         | Full workflow: build → fix symlinks → generate AppImage     |
| `appimage-build/post_build_fix.sh`        | Fix symlinks + attempt AppImage generation                  |
| `appimage-build/generate_appimage.sh`     | Standalone AppImage generation using cached plugin          |
| `appimage-build/fix_appimage_symlinks.sh` | Legacy: only fixes symlinks (use post_build_fix.sh instead) |

## How It Works

The solution uses three key techniques:

### 1. Symlink Fixing

Converts absolute symlinks to relative:

```bash
# Before
/home/thefable/Bureau/Zephyr/.../Zephyr.png

# After
Zephyr.png
```

### 2. Plugin Caching

Tauri caches linuxdeploy-plugin-appimage in `~/.cache/tauri/`. Our scripts locate and use it:

```bash
~/.cache/tauri/linuxdeploy-plugin-appimage.AppImage --appdir=./Zephyr.AppDir
```

### 3. Fallback Generation

If Tauri build fails to generate AppImage, our scripts can do it manually using the cached plugin.

## Verification

After building, verify symlinks are relative:

```bash
# Check specific symlinks
readlink src-tauri/target/release/bundle/appimage/Zephyr.AppDir/.DirIcon
# Output should be: Zephyr.png (relative, not absolute path)

# No absolute symlinks should exist
find src-tauri/target/release/bundle/appimage/Zephyr.AppDir -type l \
  -exec sh -c 'readlink "$1" | grep -q "^/" && echo "ABSOLUTE: $1"' _ {} \;
# Should output nothing
```

## AppImage Output

After successful build:

```
src-tauri/target/release/bundle/appimage/Zephyr-x86_64.AppImage
```

Size: ~104MB (varies with content updates)

## Distribution

Your AppImage is now fully portable:

- ✅ Relative symlinks work on any system
- ✅ No absolute paths
- ✅ Self-contained binary
- ✅ No dependencies required from end-user system

Simply distribute `Zephyr-x86_64.AppImage` to users!

## Build Dependencies

Required on build system:

- `pnpm` / `npm`
- `cargo` / Rust
- `gcc` / build-essential
- `linuxdeploy` (in /usr/local/bin or auto-cached)
- `fuse2` (for AppImage execution)
- `appstream` (for metadata)

All are checked and installed automatically by the build scripts.

## Troubleshooting

### "AppDir not found"

```bash
pnpm tauri build  # Create AppDir first
```

### "Plugin not found"

```bash
pnpm tauri build                       # Triggers plugin download to cache
pnpm tauri:appimage:generate           # Uses cached plugin
```

Or directly:

```bash
pnpm tauri build
./appimage-build/generate_appimage.sh
```

### Symlink issues

```bash
pnpm tauri:appimage:fix  # Manually fix symlinks
```

Or directly:

```bash
./appimage-build/post_build_fix.sh
```

### Check what's being generated

```bash
ls -la src-tauri/target/release/bundle/appimage/
find src-tauri/target/release/bundle/appimage/Zephyr.AppDir -type l
```

## Technical Details

### Absolute vs Relative Symlinks

**Absolute symlinks** include the full path:

```
/home/user/project/folder/file
```

- Only work on the exact machine that created them
- Break when AppImage is moved/distributed

**Relative symlinks** use relative references:

```
../folder/file
```

- Work on any machine
- Portable across distributions
- Perfect for AppImage

### Why This Matters

AppImages are files that contain a full Linux application environment. When packaged with absolute symlinks, those links expect files at the hardcoded path - which won't exist on a user's system.

Relative symlinks maintain the internal structure relationships, making the AppImage truly portable.

## CI/CD Integration

For automated builds, use:

```yaml
# Example GitHub Actions
- name: Build AppImage
  run: pnpm tauri:build:complete

- name: Upload AppImage
  uses: actions/upload-artifact@v3
  with:
    path: src-tauri/target/release/bundle/appimage/*.AppImage
```

Or run the script directly:

```yaml
- name: Build AppImage
  run: ./appimage-build/build_and_fix.sh

- name: Upload AppImage
  uses: actions/upload-artifact@v3
  with:
    path: src-tauri/target/release/bundle/appimage/*.AppImage
```

## Further Notes

- Scripts are idempotent - safe to run multiple times
- No files are modified except symlink targets
- All changes are reversible
- Tests on multiple systems recommended before wide distribution
