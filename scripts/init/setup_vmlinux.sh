#!/bin/bash

# Determine project root relative to this script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
PROJECT_ROOT="$SCRIPT_DIR/../.."

# Check if vmlinux.h exists in the current directory
if [ -f "$PROJECT_ROOT/landscape-ebpf/src/bpf/vmlinux.h" ]; then
  echo "vmlinux.h already exists."
else
    # Prompt the user whether to generate vmlinux.h
    read -p "vmlinux.h does not exist. Do you want to generate it? (y/n): " choice
    if [ "$choice" == "y" ]; then
        # Check if bpftool is installed
        if ! command -v bpftool &> /dev/null; then
          echo "bpftool is not installed. Please install bpftool and run this script again."
          exit 1
        fi

        # Generate vmlinux.h
        bpftool btf dump file /sys/kernel/btf/vmlinux format c > "$PROJECT_ROOT/landscape-ebpf/src/bpf/vmlinux.h"

        # Check if generation was successful
        if [ -f "$PROJECT_ROOT/landscape-ebpf/src/bpf/vmlinux.h" ]; then
          echo "vmlinux.h generated successfully."
        else
          echo "Failed to generate vmlinux.h."
          exit 1
        fi
    else
        echo "vmlinux.h generation skipped."
    fi
fi
