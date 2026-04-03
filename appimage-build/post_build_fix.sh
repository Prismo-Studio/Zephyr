#!/bin/bash

# Post-build script to fix AppImage symlinks and generate AppImage if needed
# Run this after: pnpm tauri build (even if there were build warnings/errors)

# Path to the AppImage.AppDir directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"
APPDIR="$SCRIPT_DIR/src-tauri/target/release/bundle/appimage/Zephyr.AppDir"
APPIMAGE_DIR="$SCRIPT_DIR/src-tauri/target/release/bundle/appimage"

if [ ! -d "$APPDIR" ]; then
    echo "Error: AppDir not found at $APPDIR"
    echo "Make sure to run 'pnpm tauri build' first"
    exit 1
fi

echo "🔧 Fixing AppImage symlinks in $APPDIR"

# Function to convert absolute symlinks to relative
fix_symlink() {
    local link=$1
    if [ -L "$link" ]; then
        local target=$(readlink "$link")
        
        # Check if it's an absolute symlink pointing within AppDir
        if [[ "$target" =~ ^$APPDIR ]]; then
            # Make it relative
            local relative_target="${target#$APPDIR/}"
            relative_target="${relative_target#/}"
            rm "$link"
            ln -s "$relative_target" "$link"
            echo "  ✓ Fixed: $(basename "$link") → $relative_target"
        fi
    fi
}

# Fix known problem symlinks
fix_symlink "$APPDIR/.DirIcon"
fix_symlink "$APPDIR/Zephyr.desktop"

# Check for any remaining absolute symlinks
echo ""
echo "🔍 Checking for remaining absolute symlinks..."
absolute_count=$(find "$APPDIR" -type l -exec sh -c 'readlink "$1" | grep -q "^/" && echo 1' _ {} \; 2>/dev/null | wc -l)

if [ "$absolute_count" -eq 0 ]; then
    echo "✅ All symlinks are relative!"
else
    echo "⚠️  Found $absolute_count absolute symlinks remaining:"
    find "$APPDIR" -type l -exec sh -c 'target=$(readlink "$1"); [[ "$target" =~ ^/ ]] && echo "  - $1 → $target"' _ {} \; 2>/dev/null
    echo ""
    echo "These may be okay if they're outside the AppDir scope."
fi

# Check if AppImage was already created
echo ""
APPIMAGE_FILE=$(find "$APPIMAGE_DIR" -maxdepth 1 -name "*.AppImage" -type f 2>/dev/null | head -1)
if [ -f "$APPIMAGE_FILE" ]; then
    echo "✅ AppImage already exists: $(basename "$APPIMAGE_FILE")"
    exit 0
fi

# If no AppImage exists, try to generate it using the linuxdeploy plugin
echo ""
echo "📦 Generating AppImage using linuxdeploy-plugin-appimage..."
PLUGIN="$HOME/.cache/tauri/linuxdeploy-plugin-appimage.AppImage"

if [ ! -f "$PLUGIN" ]; then
    echo "⚠️  Plugin not found at $PLUGIN"
    echo "AppImage generation skipped - it may have been created by tauri build"
    exit 0
fi

cd "$APPIMAGE_DIR"

# Ensure lowercase icon if needed
if [ -f "Zephyr.AppDir/Zephyr.png" ] && [ ! -f "Zephyr.AppDir/zephyr.png" ]; then
    cp "Zephyr.AppDir/Zephyr.png" "Zephyr.AppDir/zephyr.png"
    echo "  ℹ️  Created lowercase icon copy"
fi

# Run the plugin
"$PLUGIN" --appdir=./Zephyr.AppDir
PLUGIN_EXIT=$?

if [ $PLUGIN_EXIT -eq 0 ]; then
    echo "✅ AppImage generated successfully!"
    APPIMAGE_FILE=$(find "$APPIMAGE_DIR" -maxdepth 1 -name "*.AppImage" -type f 2>/dev/null | head -1)
    if [ -n "$APPIMAGE_FILE" ]; then
        echo "  📍 Location: $APPIMAGE_FILE"
        echo "  📏 Size: $(ls -lh "$APPIMAGE_FILE" | awk '{print $5}')"
    fi
else
    echo "⚠️  Plugin returned exit code $PLUGIN_EXIT"
    echo "The AppImage may not have been generated."
fi

exit 0
