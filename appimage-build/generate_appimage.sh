#!/bin/bash

# Complete AppImage build without relying on tauri bundle finishing
# This is a fallback when pnpm tauri build has issues with linuxdeploy

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"
APPDIR="$SCRIPT_DIR/src-tauri/target/release/bundle/appimage/Zephyr.AppDir"
APPIMAGE_DIR="$SCRIPT_DIR/src-tauri/target/release/bundle/appimage"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  📦 Manual AppImage Builder (Post-Tauri)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if AppDir exists
if [ ! -d "$APPDIR" ]; then
    echo "❌ Error: AppDir not found at $APPDIR"
    echo ""
    echo "Make sure to run 'pnpm tauri build' first to generate the AppDir structure."
    echo "The actual AppImage generation may fail, but the AppDir should still be created."
    exit 1
fi

echo "✓ Found AppDir at: $APPDIR"
echo ""

# Get the plugin
PLUGIN="$HOME/.cache/tauri/linuxdeploy-plugin-appimage.AppImage"

if [ ! -f "$PLUGIN" ]; then
    echo "❌ Error: linuxdeploy plugin not found at $PLUGIN"
    echo ""
    echo "The plugin should be cached by Tauri. Try running 'pnpm tauri build' first."
    exit 1
fi

echo "✓ Found plugin: $(basename $PLUGIN)"
echo "  Size: $(ls -lh $PLUGIN | awk '{print $5}')"
echo ""

# Fix symlinks
echo "🔧 Fixing symlinks in AppDir..."
fix_symlink() {
    local link=$1
    if [ -L "$link" ]; then
        local target=$(readlink "$link")
        if [[ "$target" =~ ^$APPDIR ]]; then
            local relative_target="${target#$APPDIR/}"
            relative_target="${relative_target#/}"
            rm "$link"
            ln -s "$relative_target" "$link"
            echo "  ✓ Fixed: $(basename "$link") → $relative_target"
        fi
    fi
}

fix_symlink "$APPDIR/.DirIcon"
fix_symlink "$APPDIR/Zephyr.desktop"

# Ensure icon exists in lowercase (some systems need it)
if [ -f "$APPDIR/Zephyr.png" ] && [ ! -f "$APPDIR/zephyr.png" ]; then
    cp "$APPDIR/Zephyr.png" "$APPDIR/zephyr.png"
    echo "  ✓ Created lowercase icon copy"
fi

echo ""
echo "📦 Generating AppImage..."
echo "  Command: $PLUGIN --appdir=./Zephyr.AppDir"
echo ""

cd "$APPIMAGE_DIR"
"$PLUGIN" --appdir=./Zephyr.AppDir
PLUGIN_EXIT=$?

echo ""

if [ $PLUGIN_EXIT -eq 0 ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  ✅ AppImage Generated Successfully!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    APPIMAGE_FILE=$(find "$APPIMAGE_DIR" -maxdepth 1 -name "*.AppImage" -type f 2>/dev/null | head -1)
    if [ -n "$APPIMAGE_FILE" ]; then
        SIZE=$(ls -lh "$APPIMAGE_FILE" | awk '{print $5}')
        echo "📍 Location: $(basename "$APPIMAGE_FILE")"
        echo "📏 Size: $SIZE"
        echo ""
        echo "✨ AppImage is ready for distribution!"
        echo "   Relative symlinks ensure portability across systems."
    fi
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  ⚠️  AppImage Generation Failed (exit code: $PLUGIN_EXIT)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi
