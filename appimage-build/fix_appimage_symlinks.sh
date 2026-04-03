#!/bin/bash

# Fix AppImage symlinks to use relative paths instead of absolute
# This ensures the AppImage works correctly when distributed

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"
APPDIR="$SCRIPT_DIR/src-tauri/target/release/bundle/appimage/Zephyr.AppDir"

if [ ! -d "$APPDIR" ]; then
    echo "AppDir not found at $APPDIR"
    exit 1
fi

echo "Converting absolute symlinks to relative symlinks in $APPDIR"

# Convert .DirIcon symlink (from AppDir root to Zephyr.png)
if [ -L "$APPDIR/.DirIcon" ]; then
    rm "$APPDIR/.DirIcon"
    ln -s "Zephyr.png" "$APPDIR/.DirIcon"
    echo "✓ Fixed .DirIcon symlink"
fi

# Convert Zephyr.desktop symlink (from AppDir root to usr/share/applications/Zephyr.desktop)
if [ -L "$APPDIR/Zephyr.desktop" ]; then
    rm "$APPDIR/Zephyr.desktop"
    ln -s "usr/share/applications/Zephyr.desktop" "$APPDIR/Zephyr.desktop"
    echo "✓ Fixed Zephyr.desktop symlink"
fi

# Check for any other absolute symlinks in AppDir
echo ""
echo "Checking for remaining absolute symlinks..."
absolute_symlinks=$(find "$APPDIR" -type l -exec sh -c 'readlink "$1" | grep -q "^/" && echo "$1"' _ {} \;)

if [ -z "$absolute_symlinks" ]; then
    echo "✓ No absolute symlinks found - AppImage is ready for distribution!"
else
    echo "Found these absolute symlinks:"
    echo "$absolute_symlinks"
    echo "Please review and fix them manually if needed."
    exit 1
fi
