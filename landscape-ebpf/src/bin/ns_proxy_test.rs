fn main() {
    // ip netns add tpns
    // ip link add veth0 type veth peer name veth1
    // ip link set veth1 netns tpns
    // ip link set veth0 up
    // ip netns exec tpns ip link set veth1 up
    // ip netns exec tpns ip link set lo up
    // ip netns exec tpns ip neigh replace 169.254.0.1 lladdr be:25:85:83:00:0d dev veth1
    // ip netns exec tpns  ip addr add 169.254.0.11/32 dev veth1
    // ip netns exec tpns  ip route add 169.254.0.1 dev veth1 scope link
    // ip netns exec tpns  ip route add default via 169.254.0.1 dev veth1

    // ip netns exec tpns ip rule add fwmark 0x1/0x1 lookup 100
    // ip netns exec tpns ip route add local default dev lo table 100
    // ip netns exec tpns sysctl -w net.ipv4.conf.lo.accept_local=1
    // ip netns exec tpns ip route add 169.254.0.1 dev veth1

    // sysctl net.ipv4.conf.veth0.proxy_arp=1
    //  sysctl net.ipv4.conf.veth0.rp_filter=2

    // curl -vvv 223.5.5.5.5:2234
    // docker run --rm -p 2234:80 --name temp-nginx nginx
    //
    // nc -lu 2235
    // nc -u 223.5.5.5.5 2235
    landscape_ebpf::ns_proxy::run()
}
