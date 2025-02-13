#!/bin/bash

dockerd &

# 创建网络命名空间 pc1 和 pc2
ip netns add pc1
ip netns add pc2

# 为 pc1 配置 veth 对
ip link add pc1 type veth peer name pc1-peer
ip link set pc1-peer netns pc1
ip link set pc1 up

# 启动 pc1 命名空间内的网卡（包括 lo）
ip netns exec pc1 ip link set lo up
ip netns exec pc1 ip link set pc1-peer up

# 在 pc1 命名空间中后台运行 dhclient（针对 pc1-peer 接口）
ip netns exec pc1 dhclient pc1-peer &> /dev/null &  # 后台运行并静默输出

# 为 pc2 配置 veth 对
ip link add pc2 type veth peer name pc2-peer
ip link set pc2-peer netns pc2
ip link set pc2 up

# 启动 pc2 命名空间内的网卡（包括 lo）
ip netns exec pc2 ip link set lo up
ip netns exec pc2 ip link set pc2-peer up

# 在 pc2 命名空间中后台运行 dhclient（针对 pc2-peer 接口）
ip netns exec pc2 dhclient pc2-peer &> /dev/null &  # 后台运行并静默输出

# 输出验证信息
echo "--------------------------------------------"
echo "[宿主机] 网卡列表:"
ip link show pc1
ip link show pc2

echo "--------------------------------------------"
echo "[命名空间 pc1] 网卡状态和 IP 地址:"
ip netns exec pc1 ip link show
ip netns exec pc1 ip addr show pc1-peer  # 显示 DHCP 获取的 IP

echo "--------------------------------------------"
echo "[命名空间 pc2] 网卡状态和 IP 地址:"
ip netns exec pc2 ip link show
ip netns exec pc2 ip addr show pc2-peer  # 显示 DHCP 获取的 IP

echo "nameserver 127.0.0.1" > /etc/resolv.conf

# 启动其他服务（如 Web 服务器）
/root/landscape-webserver
