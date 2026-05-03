#!/bin/bash
set -e

# Format Rust code
echo "Formatting Rust code..."
cargo fmt

# Format C/eBPF code (skip vmlinux.h)
echo "Formatting C/eBPF code..."
if ! command -v clang-format &>/dev/null; then
    echo "Error: clang-format not found. Install it with: apt install clang-format" >&2
    exit 1
fi
find landscape-ebpf/src/bpf -type f \( -name '*.c' -o -name '*.h' \) \
    ! -name 'vmlinux.h' \
    -print0 | xargs -0 clang-format -i

# Format Frontend code (excluding landscape-types)
echo "Formatting webui code (landscape-webui/)..."
if [ -d "landscape-webui" ]; then
    (cd landscape-webui && npx prettier --write .)
fi

echo "Format complete."
