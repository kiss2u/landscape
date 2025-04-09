#include "vmlinux.h"

#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#include "landscape.h"
#include "flow_mark_share.h"

char LICENSE[] SEC("license") = "Dual BSD/GPL";

const volatile int current_eth_net_offset = 14;

static __always_inline int current_pkg_type(struct __sk_buff *skb, int current_eth_net_offset,
                                            bool *is_ipv4_) {
    bool is_ipv4;
    if (current_eth_net_offset != 0) {
        struct ethhdr *eth;
        if (VALIDATE_READ_DATA(skb, &eth, 0, sizeof(*eth))) {
            return TC_ACT_UNSPEC;
        }

        if (eth->h_proto == ETH_IPV4) {
            is_ipv4 = true;
        } else if (eth->h_proto == ETH_IPV6) {
            is_ipv4 = false;
        } else {
            return TC_ACT_UNSPEC;
        }
    } else {
        u8 *p_version;
        if (VALIDATE_READ_DATA(skb, &p_version, 0, sizeof(*p_version))) {
            return TC_ACT_UNSPEC;
        }
        u8 ip_version = (*p_version) >> 4;
        if (ip_version == 4) {
            is_ipv4 = true;
        } else if (ip_version == 6) {
            is_ipv4 = false;
        } else {
            return TC_ACT_UNSPEC;
        }
    }
    *is_ipv4_ = is_ipv4;
    return TC_ACT_OK;
}

SEC("tc/ingress")
int flow_ingress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC ">> flow_ingress"
    bool is_ipv4;

    struct flow_match_key find_flow_key = {0};
    int ret;
    if (current_pkg_type(skb, current_eth_net_offset, &is_ipv4) != TC_ACT_OK) {
        return TC_ACT_UNSPEC;
    }

    if (current_eth_net_offset > 0) {
        ret = bpf_skb_load_bytes(skb, 6, &find_flow_key.h_source, sizeof(find_flow_key.h_source));
        if (ret) {
            bpf_log_info("mac bpf_skb_load_bytes error");
            return TC_ACT_SHOT;
        }
    }

    if (is_ipv4) {
        ret = bpf_skb_load_bytes(skb, current_eth_net_offset + 1, &find_flow_key.tos,
                                 sizeof(find_flow_key.tos));
        if (ret) {
            bpf_log_info("ipv4 bpf_skb_load_bytes error");
            return TC_ACT_SHOT;
        }
    } else {
        __u16 first_2_bytes;
        ret =
            bpf_skb_load_bytes(skb, current_eth_net_offset, &first_2_bytes, sizeof(first_2_bytes));
        if (ret) {
            bpf_log_info("ipv6 bpf_skb_load_bytes error");
            return TC_ACT_SHOT;
        }
        first_2_bytes = bpf_ntohs(first_2_bytes);
        find_flow_key.tos = (first_2_bytes >> 4) & 0xFF;
    }

    // find_flow_key.vlan_tci = skb->vlan_present;
    PRINT_MAC_ADDR(find_flow_key.h_source);
    bpf_log_info("tos: %d", find_flow_key.tos);
    bpf_log_info("vlan_tci: %d", find_flow_key.vlan_tci);
    u32 *flow_id = bpf_map_lookup_elem(&flow_match_map, &find_flow_key);
    if (flow_id == NULL || *flow_id == 0) {
        // 查不到 flow 配置，通过使用默认路由进行处理
        return TC_ACT_UNSPEC;
    }

    skb->mark = *flow_id;
    bpf_log_info("flow_id: %d", *flow_id);

    struct bpf_fib_lookup fib_params = {0};
    fib_params.ifindex = skb->ifindex;
    fib_params.family = is_ipv4 ? AF_INET : AF_INET6;
    fib_params.sport = 0;
    fib_params.dport = 0;

    if (is_ipv4) {
        struct iphdr iph;

        // 读取 IPv4 头部
        ret = bpf_skb_load_bytes(skb, current_eth_net_offset, &iph, sizeof(iph));
        if (ret) {
            bpf_log_info("ipv4 bpf_skb_load_bytes error");
            return TC_ACT_SHOT;
        }

        // 填充协议与地址
        fib_params.l4_protocol = iph.protocol;
        fib_params.ipv4_src = iph.saddr;
        fib_params.ipv4_dst = iph.daddr;
    } else {
        struct ipv6hdr ip6h;

        // 读取 IPv6 头部
        ret = bpf_skb_load_bytes(skb, current_eth_net_offset, &ip6h, sizeof(ip6h));
        if (ret) {
            bpf_log_info("ipv6 bpf_skb_load_bytes error");
            return TC_ACT_SHOT;
        }

        // 填充协议与地址
        fib_params.l4_protocol = ip6h.nexthdr;
        COPY_ADDR_FROM(fib_params.ipv6_src, ip6h.saddr.in6_u.u6_addr32);
        COPY_ADDR_FROM(fib_params.ipv6_dst, ip6h.daddr.in6_u.u6_addr32);
    }

    // TODO：先检查下目标 IP 是不是在黑名单中

    int rc = bpf_fib_lookup(skb, &fib_params, sizeof(fib_params), 0);

    if (rc == BPF_FIB_LKUP_RET_NOT_FWDED) {
        // 发往本机的直接放行
        return TC_ACT_UNSPEC;
    }
    // 不是发往本机的
    bpf_log_info("other");
    // 1. 先检查有没有额外的 DNS 配置

    // 2. 根据 flow_id 检索 flow 的配置

    // 3. 依据配置发往具体的端口

    return TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}

SEC("tc/egress")
int flow_egress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC ">> flow_egress"
    struct ethhdr *eth;
    if (VALIDATE_READ_DATA(skb, &eth, 0, sizeof(*eth))) {
        return TC_ACT_UNSPEC;
    }

    bpf_log_info("mark: %d", skb->mark);
    bpf_log_info("ifindex: %d", skb->ifindex);
    bpf_log_info("ingress_ifindex: %d", skb->ingress_ifindex);

    return TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}