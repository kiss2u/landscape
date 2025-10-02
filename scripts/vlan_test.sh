#!/bin/bash

# 1. 创建命名空间
ip netns add vlanns

# 2. 创建 veth pair
ip link add vveth0 type veth peer name vveth1

# 3. 把 vveth1 移动到命名空间 vlanns
ip link set vveth1 netns vlanns

# 4. 启动主命名空间的 vveth0
ip link set vveth0 up

# 5. 在主命名空间的 vveth0 上创建 VLAN 100
ip link add link vveth0 name vveth0.100 type vlan id 100
ip addr add 10.110.1.1/24 dev vveth0.100
ip link set vveth0.100 up

# 6. 在 vlanns 里面操作 vveth1
ip netns exec vlanns ip link set vveth1 up

# 7. 在 vlanns 里面的 vveth1 上创建 VLAN 100
ip netns exec vlanns ip link add link vveth1 name vveth1.100 type vlan id 100
ip netns exec vlanns ip addr add 10.110.1.2/24 dev vveth1.100
ip netns exec vlanns ip link set vveth1.100 up
