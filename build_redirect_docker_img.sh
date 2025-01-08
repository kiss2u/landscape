#!/bin/bash

echo "Building the project..."
cargo build --release --package landscape-ebpf --bin redirect_demo_server --bin redirect_pkg_handler
if [ $? -ne 0 ]; then
    echo "Build failed. Exiting."
    exit 1
fi

# 创建构建目录
COPY_DIR="./dockerfiles/redirect_prog/apps"
BUILD_DIR="./dockerfiles/redirect_prog"
echo "Creating build directory: $COPY_DIR"
rm -rf "$COPY_DIR" # 清理旧的构建目录
mkdir -p "$COPY_DIR"

# 复制编译产物
echo "Copying build artifacts..."
cp "$BUILD_DIR/start.sh" "$COPY_DIR/start.sh"
chmod +x "$COPY_DIR/start.sh"

cp ./target/release/redirect_demo_server "$COPY_DIR/redirect_demo_server"
cp ./target/release/redirect_pkg_handler "$COPY_DIR/redirect_pkg_handler"
chmod +x "$COPY_DIR/redirect_demo_server"
chmod +x "$COPY_DIR/redirect_pkg_handler"


# 进入构建目录并构建 Docker 镜像
cd "$BUILD_DIR"
echo "Building Docker image..."
docker build -t ld_redirect_image .
if [ $? -ne 0 ]; then
    echo "Docker build failed. Exiting."
    exit 1
fi

echo "Docker image built successfully: ld_redirect_image"
