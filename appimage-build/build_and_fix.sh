#!/bin/bash

# Complete AppImage build workflow with symlink fixes
# Uses fallback manual generation if tauri build fails

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"
APPDIR="$SCRIPT_DIR/src-tauri/target/release/bundle/appimage/Zephyr.AppDir"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🚀 Zephyr AppImage Build & Fix Workflow"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Step 1: Build
echo "📦 Building Zephyr AppImage..."
echo ""
cd "$SCRIPT_DIR"
pnpm tauri build
BUILD_EXIT_CODE=$?

if [ $BUILD_EXIT_CODE -ne 0 ]; then
    echo ""
    echo "⚠️  Build encountered issues (exit code: $BUILD_EXIT_CODE)"
fi

# Check if AppDir was created (even if build had warnings/errors)
if [ ! -d "$APPDIR" ]; then
    echo "❌ Critical: AppDir not found at $APPDIR"
    echo "   Build failed before creating application structure."
    exit 1
fi

echo ""
echo "✅ AppDir created successfully!"
echo ""

# Step 2: Fix symlinks and try to generate AppImage
echo "🔧 Fixing AppImage symlinks and generating AppImage..."
echo ""
"$SCRIPT_DIR/appimage-build/post_build_fix.sh"
FIX_EXIT_CODE=$?

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Build Process Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Step 3: Summary
APPIMAGE_DIR="$SCRIPT_DIR/src-tauri/target/release/bundle/appimage"
echo "📍 Build Output:"
echo "  AppDir: $APPDIR"

# Show AppImage if it was created
APPIMAGE_FILE=$(find "$APPIMAGE_DIR" -maxdepth 1 -name "*.AppImage" -type f 2>/dev/null | head -1)
if [ -n "$APPIMAGE_FILE" ]; then
    echo "  AppImage: $(basename "$APPIMAGE_FILE")"
    echo "            $(ls -lh "$APPIMAGE_FILE" | awk '{print $5}')"
    echo ""
    echo "✨ AppImage is ready for distribution!"
    echo "   Symlinks are relative and portable across systems."
else
    echo ""
    echo "⚠️  No AppImage found. Try running manually:"
    echo "    ./generate_appimage.sh"
fi

echo ""
if [ $BUILD_EXIT_CODE -eq 0 ]; then
    echo "✅ Build completed successfully!"
else
    echo "⚠️  Build had issues, but AppDir was created."
    echo "   Review the output above for details."
fi

echo ""

exit $BUILD_EXIT_CODE
