#!/bin/bash
set -e

# This script should be run from the project root
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

BASE_DIR=$(pwd)
LIB_DIR="$BASE_DIR/libduckdb"

# Extract DuckDB version from Cargo.toml
# Look for line like: duckdb = { version = "1.4.4" }
DUCKDB_VERSION=$(grep -E 'duckdb[[:space:]]*=[[:space:]]*\{[[:space:]]*version[[:space:]]*=[[:space:]]*"[^"]+"' Cargo.toml | sed -E 's/.*version = "([^"]+)".*/\1/')

if [ -z "$DUCKDB_VERSION" ]; then
    # Try workspace dependencies format if not found in root dependencies
    DUCKDB_VERSION=$(grep -E 'duckdb[[:space:]]*=[[:space:]]*"[^"]+"' Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')
fi

if [ -z "$DUCKDB_VERSION" ]; then
    echo "Error: Could not find duckdb version in Cargo.toml. Please check the file format."
    exit 1
fi

echo "Detected DuckDB version: $DUCKDB_VERSION"

# Detection architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        DUCKDB_ARCH="amd64"
        ;;
    aarch64)
        DUCKDB_ARCH="arm64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

URL="https://github.com/duckdb/duckdb/releases/download/v$DUCKDB_VERSION/libduckdb-linux-$DUCKDB_ARCH.zip"
ZIP_FILE="libduckdb-linux-$DUCKDB_ARCH.zip"

echo "Downloading DuckDB $DUCKDB_VERSION for $DUCKDB_ARCH..."
if ! wget -q --show-progress "$URL" -O "$ZIP_FILE"; then
    echo "Error: Failed to download DuckDB from $URL. Please check your internet connection."
    exit 1
fi

echo "Unzipping to $LIB_DIR..."
mkdir -p "$LIB_DIR"
unzip -o "$ZIP_FILE" -d "$LIB_DIR"
rm "$ZIP_FILE"

# Create environment file for easy sourcing
ENV_FILE="$BASE_DIR/duckdb.env"
cat <<EOF > "$ENV_FILE"
# DuckDB Local Environment Configuration
export DUCKDB_LIB_DIR="$LIB_DIR"
export DUCKDB_INCLUDE_DIR="$LIB_DIR"
export LD_LIBRARY_PATH="\$LD_LIBRARY_PATH:$LIB_DIR"
# Unset DUCKDB_DOWNLOAD_LIB to avoid attempted downloads
unset DUCKDB_DOWNLOAD_LIB
EOF

chmod +x "$ENV_FILE"

echo "------------------------------------------------"
echo "Setup successful!"
echo "DuckDB library is located at: $LIB_DIR"
echo ""
echo "To apply environment variables in your current shell, run:"
echo "  source ./duckdb.env"
echo ""
echo "Or use ./dev.sh which has been updated to use the local library."
echo "------------------------------------------------"

# Automatically update dev.sh if it exists to prefer local duckdb
if [ -f "dev.sh" ]; then
    echo "Updating dev.sh to prefer local DuckDB if available..."
    cat <<'INNEREOF' > dev.sh
#!/usr/bin/env bash

# Use local DuckDB if it exists, otherwise fall back to download
if [ -d "./libduckdb" ]; then
    export DUCKDB_LIB_DIR="$(pwd)/libduckdb"
    export DUCKDB_INCLUDE_DIR="$DUCKDB_LIB_DIR"
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$DUCKDB_LIB_DIR"
    unset DUCKDB_DOWNLOAD_LIB
else
    export DUCKDB_DOWNLOAD_LIB=1
fi

# Run cargo with any arguments passed to this script
cargo run "$@"
INNEREOF
    chmod +x dev.sh
fi
