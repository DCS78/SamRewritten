#!/bin/bash
#
# rebuild_arch.sh - Developer script to build the Arch Linux package for SamRewritten.
#
# Usage: Run from a system with makepkg and PKGBUILD available.
# Note: Building in a bind-mounted volume is not supported; this script builds in /tmp and copies the result.

set -euo pipefail

PKG_SRC="/mnt/package/PKGBUILD"
BUILD_DIR="/tmp/build"
DEST_DIR="/mnt/package"

# Check for PKGBUILD
if [[ ! -f "$PKG_SRC" ]]; then
	echo "Error: PKGBUILD not found at $PKG_SRC" >&2
	exit 1
fi

# Prepare build directory
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"
cp "$PKG_SRC" "$BUILD_DIR"

# Build package
pushd "$BUILD_DIR" > /dev/null
makepkg
popd > /dev/null

# Copy built package(s) to destination
cp "$BUILD_DIR"/*.zst "$DEST_DIR"