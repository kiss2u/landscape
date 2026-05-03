#!/bin/bash
set -e

# Format Rust code
echo "Formatting Rust code..."
cargo fmt

# Format C/eBPF code (skip vmlinux.h)
# clang-format 18 is required for CI compatibility.
# Install: apt install clang-format-18
echo "Formatting C/eBPF code..."
CLANG_FORMAT=""
if command -v clang-format-18 &>/dev/null; then
    CLANG_FORMAT="clang-format-18"
elif command -v clang-format &>/dev/null; then
    CF_VERSION=$(clang-format --version | grep -oP '\d+' | head -1)
    if [ "$CF_VERSION" = "18" ]; then
        CLANG_FORMAT="clang-format"
    else
        echo "Error: clang-format version $CF_VERSION found, but version 18 is required." >&2
        echo "CI uses clang-format-18, other versions may produce different results." >&2
        echo "Install: apt install clang-format-18" >&2
        exit 1
    fi
else
    echo "Error: clang-format not found. Install clang-format-18:" >&2
    echo "  apt install clang-format-18" >&2
    exit 1
fi
echo "Using $($CLANG_FORMAT --version)"
find landscape-ebpf/src/bpf -type f \( -name '*.c' -o -name '*.h' \) \
    ! -name 'vmlinux.h' \
    -print0 | xargs -0 "$CLANG_FORMAT" -i

# Format Frontend code (excluding landscape-types)
echo "Formatting webui code (landscape-webui/)..."
if [ -d "landscape-webui" ]; then
    (cd landscape-webui && npx prettier --write .)
fi

echo "Format complete."
