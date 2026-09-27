#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== Initializing Landscape Development Environment ==="

# Run vmlinux setup
bash "$SCRIPT_DIR/init/setup_vmlinux.sh"

echo "=== Initialization Complete! ==="
