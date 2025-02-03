# 使用 Debian 作为基础镜像
FROM debian:latest

# 安装必要的工具和依赖
RUN apt-get update && \
    apt-get install -y \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg \
    lsb-release \
    software-properties-common \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# 添加 Docker 的官方 GPG 密钥
RUN curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

# 添加 Docker 的稳定版仓库
RUN echo \
    "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/debian \
    $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null


# 设置工作目录
WORKDIR /root

# 安装必要的工具
RUN apt-get update && apt-get install -y \
    iproute2 \
    docker-ce docker-ce-cli containerd.io  \
    curl  \
    isc-dhcp-client \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY apps/ ./

# 使用启动脚本作为容器启动命令
CMD ["/root/start.sh"]
