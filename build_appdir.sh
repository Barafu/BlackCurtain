#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APPDIR="$SCRIPT_DIR/AppDir"

# Extract version from Cargo.toml
VERSION=$(grep -m1 '^version' "$SCRIPT_DIR/Cargo.toml" | sed -E 's/.*=\s*"([^"]+)".*/\1/')
EXEC="black_curtain"

echo "Building AppDir for version $VERSION"

# Clean and recreate AppDir
rm -rf "$APPDIR"
mkdir -p "$APPDIR"

# Copy binary
mkdir -p "$APPDIR/usr/bin"
cp "$SCRIPT_DIR/target/release/$EXEC" "$APPDIR/usr/bin/barafu-black-curtain"

# Copy icons
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
cp "$SCRIPT_DIR/data/icon256.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/barafu-black-curtain.png"

mkdir -p "$APPDIR/usr/share/icons/hicolor/48x48/apps"
cp "$SCRIPT_DIR/data/icon48.png" "$APPDIR/usr/share/icons/hicolor/48x48/apps/barafu-black-curtain.png"

# Generate .desktop file from template
mkdir -p "$APPDIR/usr/share/applications"
sed -e "s/{VERSION}/$VERSION/g" -e "s/{EXEC}/barafu-black-curtain/g" \
    "$SCRIPT_DIR/data/barafu-black-curtain.desktop.template" \
    > "$APPDIR/usr/share/applications/barafu-black-curtain.desktop"

# Symlink icon to AppDir root
ln -s "usr/share/icons/hicolor/256x256/apps/barafu-black-curtain.png" "$APPDIR/.DirIcon"
ln -s "usr/share/icons/hicolor/256x256/apps/barafu-black-curtain.png" "$APPDIR/barafu-black-curtain.png"

# Symlink .desktop to AppDir root
ln -s "usr/share/applications/barafu-black-curtain.desktop" "$APPDIR/barafu-black-curtain.desktop"

# Create AppRun
cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/bash
exec $APPDIR/usr/bin/barafu-black-curtain
EOF
chmod +x "$APPDIR/AppRun"

echo "AppDir created at $APPDIR"
