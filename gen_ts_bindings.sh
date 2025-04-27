#!/bin/sh
rm -rf ./landscape-webui/src/rust_bindings
cargo test --workspace  export_bindings