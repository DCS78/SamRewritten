#!/bin/bash
#
# rebuild_snap.sh - Developer script to build and install the SamRewritten Snap package.
#
# Usage: Run from the project root directory.
# - Classic confinement is for production.
# - Strict with devmode is for screenshots/testing.
#
# This script removes old snaps, rebuilds, and installs the new snap.
#
# WARNING: This script will remove any existing 'samrewritten' snap installation.

set -euo pipefail

# Remove old snap files (if any)
find . -maxdepth 1 -name '*.snap' -exec rm -f {} +

# Remove existing snap if installed (ignore errors)
if snap list samrewritten &>/dev/null; then
	snap remove samrewritten || true
fi

# Build the snap
snapcraft

# Install the snap (classic confinement for production)
snap install ./*.snap --classic --dangerous

# For strict/devmode testing, uncomment one of the following lines:
# snap install --devmode --dangerous ./*.snap
# snap install --dangerous ./*.snap
# snap connect samrewritten:access-steam-folder