#!/bin/bash
set -e

SCRIPT_DIR="$(dirname "$0")"

echo "=== Initializing Landscape Development Environment ==="

# Run vmlinux setup
bash "$SCRIPT_DIR/scripts/init/setup_vmlinux.sh"

echo "=== Initialization Complete! ==="
