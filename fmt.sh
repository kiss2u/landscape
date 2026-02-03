#!/bin/bash

# Format Rust code
echo "Formatting Rust code..."
cargo fmt

# Format Frontend code (excluding landscape-types)
echo "Formatting webui code (landscape-webui/)..."
if [ -d "landscape-webui" ]; then
    (cd landscape-webui && npx prettier --write .)
fi

echo "Format complete."
