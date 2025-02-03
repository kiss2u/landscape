#!/usr/bin/env bash

echo "SCRIPT_DIR: $SCRIPT_DIR"
echo "Target platform set to: $TARGET"
echo "Rust target: $TARGET_ARCH"
echo "Docker architecture: $DOCKER_ARCH"

echo "构建 Vue 项目..."
cd "$SCRIPT_DIR/landscape-webui" || { echo "找不到 landscape-webui 目录"; exit 1; }

yarn install || { echo "Vue 项目依赖安装失败"; exit 1; }
yarn build || { echo "Vue 项目构建失败"; exit 1; }

echo "复制 Vue 构建产物到 output/static..."
mkdir -p "$SCRIPT_DIR/output/static"
cp -r dist/* "$SCRIPT_DIR/output/static/"
cd "$SCRIPT_DIR"