#!/usr/bin/env bash
#
# 准备 Docker 构建上下文
#

# 注意，这里用绝对路径，避免cd带来的混乱
COPY_DIR="$SCRIPT_DIR/dockerfiles/landscape/apps"
BUILD_DIR="$SCRIPT_DIR/dockerfiles/landscape"

echo "Creating build directory: $COPY_DIR"
rm -rf "$COPY_DIR"  # 清理旧的构建目录
mkdir -p "$COPY_DIR/.landscape-router"


# 继续复制其他文件（不受影响）
echo "Copying main artifacts..."
cp "$BUILD_DIR/start.sh" "$COPY_DIR/start.sh"
chmod +x "$COPY_DIR/start.sh"

cp "$SCRIPT_DIR/target/$TARGET_ARCH/release/landscape-webserver" \
   "$COPY_DIR/landscape-webserver"
chmod +x "$COPY_DIR/landscape-webserver"

cp -r "$SCRIPT_DIR/output/static" \
   "$COPY_DIR/.landscape-router/static"

#
# 第三阶段：使用 Buildx 构建 Docker 镜像并输出到 tar 文件
#

echo "Building Docker image for $DOCKER_ARCH..."

BUILDER_NAME="temp_builder_$(date +%s)"
# 启用 Docker Buildx (确保已启用多平台支持)
docker buildx create --use --name "$BUILDER_NAME" --driver docker-container

# 定义输出文件路径（在脚本所在目录下）
OUTPUT_DIR="$SCRIPT_DIR/output"
mkdir -p "$OUTPUT_DIR"

OUTPUT_PATH="$OUTPUT_DIR/docker_${TARGET}.tar"

# 使用 docker buildx 来构建镜像，并输出到一个 Docker-compatible .tar 文件
docker buildx build \
    --platform "linux/$DOCKER_ARCH" \
    -t "$IMAGE_NAME:$TARGET" \
    --output "type=docker,dest=$OUTPUT_PATH" \
    --build-arg CONTEXT=/mnt \
    --build-context output="$OUTPUT_DIR" \
    -f "$BUILD_DIR/Dockerfile" \
    "$BUILD_DIR"

if [ $? -ne 0 ]; then
    echo "Docker build failed. Exiting."
    exit 1
fi


docker buildx rm "$BUILDER_NAME"


echo "Docker image built successfully for $DOCKER_ARCH: ld_redirect_image:$TARGET"
echo "The tar file is at: $OUTPUT_PATH"

#
# 如果本机架构与目标架构一致，则自动 load
#

LOCAL_ARCH="$(uname -m)"
if [[ "$LOCAL_ARCH" == "$TARGET" ]]; then
    echo "Local architecture matches target. Loading image into Docker..."
    docker load -i "$OUTPUT_PATH"

    if [ $? -ne 0 ]; then
        echo "Docker load failed. Exiting."
        exit 1
    fi

    echo "Docker image loaded successfully into local Docker."
else
    echo "Local architecture does not match target."
    echo "The image tar has been saved to: $OUTPUT_PATH"
fi

echo "Done."
cd "$SCRIPT_DIR"
