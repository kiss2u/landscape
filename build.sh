#!/bin/bash

# 进入 Vue 项目目录并构建
echo "构建 Vue 项目..."
cd landscape-webui || { echo "找不到 landscape-webui 目录"; exit 1; }

# 安装依赖并构建 Vue 项目
yarn install || { echo "Vue 项目依赖安装失败"; exit 1; }
yarn build || { echo "Vue 项目构建失败"; exit 1; }

# 将 Vue 构建产物复制到 output/static
echo "复制 Vue 构建产物到 output/static..."
mkdir -p ../output/static
cp -r dist/* ../output/static/

# 返回上级目录
cd ..

# 进入 Rust 项目目录并构建
echo "构建 Rust 项目..."
# cd landscape || { echo "找不到 landscape 目录"; exit 1; }

# 构建 Rust 项目
cargo build --release || { echo "Rust 项目构建失败"; exit 1; }

# 将 Rust 构建产物复制到 output
echo "复制 Rust 构建产物到 output..."
mkdir -p ./output
cp target/release/landscape-webserver ./output/

# 完成
echo "构建和复制过程完成！"
