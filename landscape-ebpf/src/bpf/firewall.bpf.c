#include <vmlinux.h>

#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#include "landscape.h"
#include "wan_tc_pipeline.h"
#include "firewall/firewall.h"
#include "firewall/firewall_packet.h"
#include "firewall/firewall_share.h"
#include "pkg_fragment.h"

char LICENSE[] SEC("license") = "Dual BSD/GPL";

const volatile u32 current_l3_offset = 14;

#undef BPF_LOG_TOPIC

SEC("tc/egress") int ipv4_egress_firewall(struct __sk_buff *skb);
SEC("tc/ingress") int ipv4_ingress_firewall(struct __sk_buff *skb);
SEC("tc/egress") int ipv6_egress_firewall(struct __sk_buff *skb);
SEC("tc/ingress") int ipv6_ingress_firewall(struct __sk_buff *skb);

struct {
    __uint(type, BPF_MAP_TYPE_PROG_ARRAY);
    __uint(max_entries, 2);
    __uint(key_size, sizeof(u32));
    __uint(value_size, sizeof(u32));
    __array(values, int());
} ingress_prog_array SEC(".maps") = {
    .values =
        {
            [IPV4_FIREWALL_INGRESS_PROG_INDEX] = (void *)&ipv4_ingress_firewall,
            [IPV6_FIREWALL_INGRESS_PROG_INDEX] = (void *)&ipv6_ingress_firewall,
        },
};

struct {
    __uint(type, BPF_MAP_TYPE_PROG_ARRAY);
    __uint(max_entries, 2);
    __uint(key_size, sizeof(u32));
    __uint(value_size, sizeof(u32));
    __array(values, int());
} egress_prog_array SEC(".maps") = {
    .values =
        {
            [IPV4_FIREWALL_EGRESS_PROG_INDEX] = (void *)&ipv4_egress_firewall,
            [IPV6_FIREWALL_EGRESS_PROG_INDEX] = (void *)&ipv6_egress_firewall,
        },
};

SEC("tc/egress")
int ipv4_egress_firewall(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "<<< ipv4_egress_firewall <<<"

    // ld_bpf_log("bpf_tail_call ipv4_egress_firewall");

    struct packet_context packet_info;
    struct packet_offset_info offset_info = {0};
    struct inet_pair ip_pair = {0};
    __builtin_memset(&packet_info, 0, sizeof(packet_info));
    int ret = extract_firewall_v4_packet_info(skb, &packet_info, &offset_info, &ip_pair,
                                              current_l3_offset);

    if (unlikely(ret != TC_ACT_OK)) {
        if (ret == TC_ACT_SHOT) {
            ld_bpf_log("invalid packet");
        }
        return TC_ACT_UNSPEC;
    }

    bool is_icmpx_error = is_firewall_icmp_error_pkt(&packet_info);
    if (likely(!is_icmpx_error)) {
        ret = frag_info_track(&offset_info, &ip_pair);
        if (ret != TC_ACT_OK) {
            return TC_ACT_SHOT;
        }
    }

    // if (bpf_ntohs(packet_info.ip_hdr.pair_ip.src_port) == 68) {
    //     ld_bpf_log(
    //         "packet ip_protocol: %u, ip:%pI4:%u->%pI4:%u", packet_info.ip_hdr.ip_protocol,
    //         &packet_info.ip_hdr.pair_ip.src_addr, bpf_ntohs(packet_info.ip_hdr.pair_ip.src_port),
    //         &packet_info.ip_hdr.pair_ip.dst_addr,
    //         bpf_ntohs(packet_info.ip_hdr.pair_ip.dst_port));
    // }

    // ld_bpf_log(
    //     "packet ip_protocol: %u, ip:%pI4:%u->%pI4:%u", packet_info.ip_hdr.ip_protocol,
    //     &packet_info.ip_hdr.pair_ip.src_addr, bpf_ntohs(packet_info.ip_hdr.pair_ip.src_port),
    //     &packet_info.ip_hdr.pair_ip.dst_addr, bpf_ntohs(packet_info.ip_hdr.pair_ip.dst_port));
    // ld_bpf_log("packet ICMP type: %u ", packet_info.ip_hdr.icmp_type);
    struct ipv4_lpm_key block_search_key = {
        .prefixlen = 32,
        .addr = packet_info.ip_hdr.pair_ip.dst_addr.ip,
    };
    struct ipv4_mark_action *mark_value =
        bpf_map_lookup_elem(&firewall_block_ip4_map, &block_search_key);

    if (unlikely(mark_value)) {
        return TC_ACT_SHOT;
    }

    return wan_tc_pipeline_continue_egress(skb, EGRESS_STAGE_FIREWALL, TC_ACT_UNSPEC);
#undef BPF_LOG_TOPIC
}

SEC("tc/ingress")
int ipv4_ingress_firewall(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "<<< ipv4_ingress_firewall <<<"

    struct packet_context packet_info;
    struct packet_offset_info offset_info = {0};
    struct inet_pair ip_pair = {0};
    __builtin_memset(&packet_info, 0, sizeof(packet_info));
    int ret = extract_firewall_v4_packet_info(skb, &packet_info, &offset_info, &ip_pair,
                                              current_l3_offset);
    if (unlikely(ret != TC_ACT_OK)) {
        if (ret == TC_ACT_SHOT) {
            ld_bpf_log("invalid packet");
        }
        return wan_tc_pipeline_continue_ingress(skb, INGRESS_STAGE_FIREWALL, TC_ACT_UNSPEC);
    }

    bool is_icmpx_error = is_firewall_icmp_error_pkt(&packet_info);
    if (likely(!is_icmpx_error)) {
        ret = frag_info_track(&offset_info, &ip_pair);
        if (ret != TC_ACT_OK) {
            return TC_ACT_SHOT;
        }
    }

    // if (packet_info.ip_hdr.ip_protocol == IPPROTO_ICMP) {
    //     ld_bpf_log(
    //         "packet ip_protocol: %u, ip:%pI4:%u->%pI4:%u", packet_info.ip_hdr.ip_protocol,
    //         &packet_info.ip_hdr.pair_ip.src_addr, bpf_ntohs(packet_info.ip_hdr.pair_ip.src_port),
    //         &packet_info.ip_hdr.pair_ip.dst_addr,
    //         bpf_ntohs(packet_info.ip_hdr.pair_ip.dst_port));
    // }

    // ld_bpf_log("packet ip:%pI4->%pI4", &packet_info.ip_hdr.pair_ip.src_addr,
    //              &packet_info.ip_hdr.pair_ip.dst_addr);

    // ld_bpf_log("packet ip_protocol: %u ", packet_info.ip_hdr.ip_protocol);
    // ld_bpf_log("packet src port: %u ", bpf_ntohs(packet_info.ip_hdr.pair_ip.src_port));
    // ld_bpf_log("packet dst port: %u ", bpf_ntohs(packet_info.ip_hdr.pair_ip.dst_port));

    struct ipv4_lpm_key block_search_key = {
        .prefixlen = 32,
        .addr = packet_info.ip_hdr.pair_ip.src_addr.ip,
    };
    struct ipv4_mark_action *mark_value =
        bpf_map_lookup_elem(&firewall_block_ip4_map, &block_search_key);

    if (unlikely(mark_value)) {
        return TC_ACT_SHOT;
    }

    return wan_tc_pipeline_continue_ingress(skb, INGRESS_STAGE_FIREWALL, TC_ACT_UNSPEC);
#undef BPF_LOG_TOPIC
}

SEC("tc/egress")
int ipv6_egress_firewall(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "<<< ipv6_egress_firewall <<<"

    struct packet_context packet_info;
    struct packet_offset_info offset_info = {0};
    struct inet_pair ip_pair = {0};
    __builtin_memset(&packet_info, 0, sizeof(packet_info));
    int ret = extract_firewall_v6_packet_info(skb, &packet_info, &offset_info, &ip_pair,
                                              current_l3_offset);
    if (unlikely(ret != TC_ACT_OK)) {
        if (ret == TC_ACT_SHOT) {
            ld_bpf_log("invalid packet");
        }
        return wan_tc_pipeline_continue_egress(skb, EGRESS_STAGE_FIREWALL, TC_ACT_UNSPEC);
    }

    bool is_icmpx_error = is_firewall_icmp_error_pkt(&packet_info);
    if (likely(!is_icmpx_error)) {
        ret = frag_info_track(&offset_info, &ip_pair);
        if (unlikely(ret != TC_ACT_OK)) {
            return TC_ACT_SHOT;
        }
    }

    // ld_bpf_log("packet ip: [%pI6c]->[%pI6c]", &packet_info.ip_hdr.pair_ip.src_addr,
    //              &packet_info.ip_hdr.pair_ip.dst_addr);
    // ld_bpf_log("packet ip_protocol: %u ", packet_info.ip_hdr.ip_protocol);
    // ld_bpf_log("packet src port: %u ", bpf_ntohs(packet_info.ip_hdr.pair_ip.src_port));
    // ld_bpf_log("packet dst port: %u ", bpf_ntohs(packet_info.ip_hdr.pair_ip.dst_port));

    struct ipv6_lpm_key block_search_key = {
        .prefixlen = 128,
        .addr = packet_info.ip_hdr.pair_ip.dst_addr.ip,
    };
    struct firewall_action *mark_value =
        bpf_map_lookup_elem(&firewall_block_ip6_map, &block_search_key);

    if (unlikely(mark_value)) {
        return TC_ACT_SHOT;
    }

    return wan_tc_pipeline_continue_egress(skb, EGRESS_STAGE_FIREWALL, TC_ACT_UNSPEC);
#undef BPF_LOG_TOPIC
}

SEC("tc/ingress")
int ipv6_ingress_firewall(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "<<< ipv6_ingress_firewall <<<"

    struct packet_context packet_info;
    struct packet_offset_info offset_info = {0};
    struct inet_pair ip_pair = {0};
    __builtin_memset(&packet_info, 0, sizeof(packet_info));
    int ret = extract_firewall_v6_packet_info(skb, &packet_info, &offset_info, &ip_pair,
                                              current_l3_offset);
    if (unlikely(ret != TC_ACT_OK)) {
        if (ret == TC_ACT_SHOT) {
            ld_bpf_log("invalid packet");
        }
        return wan_tc_pipeline_continue_ingress(skb, INGRESS_STAGE_FIREWALL, TC_ACT_UNSPEC);
    }

    bool is_icmpx_error = is_firewall_icmp_error_pkt(&packet_info);
    if (likely(!is_icmpx_error)) {
        ret = frag_info_track(&offset_info, &ip_pair);
        if (unlikely(ret != TC_ACT_OK)) {
            return TC_ACT_SHOT;
        }
    }

    // ld_bpf_log("packet ip: [%pI6c]->[%pI6c]", &packet_info.ip_hdr.pair_ip.src_addr,
    //              &packet_info.ip_hdr.pair_ip.dst_addr);
    // ld_bpf_log("packet ip_protocol: %u ", packet_info.ip_hdr.ip_protocol);
    // ld_bpf_log("packet src port: %u ", bpf_ntohs(packet_info.ip_hdr.pair_ip.src_port));
    // ld_bpf_log("packet dst port: %u ", bpf_ntohs(packet_info.ip_hdr.pair_ip.dst_port));

    struct ipv6_lpm_key block_search_key = {
        .prefixlen = 128,
        .addr = packet_info.ip_hdr.pair_ip.src_addr.ip,
    };
    struct firewall_action *mark_value =
        bpf_map_lookup_elem(&firewall_block_ip6_map, &block_search_key);

    if (unlikely(mark_value)) {
        return TC_ACT_SHOT;
    }

    return wan_tc_pipeline_continue_ingress(skb, INGRESS_STAGE_FIREWALL, TC_ACT_UNSPEC);
#undef BPF_LOG_TOPIC
}

/// main function
SEC("tc/egress")
int egress_firewall(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "<<< egress_firewall <<<"

    bool is_ipv4;
    int ret;
    if (current_pkg_type(skb, current_l3_offset, &is_ipv4) != TC_ACT_OK) {
        return wan_tc_pipeline_continue_egress(skb, EGRESS_STAGE_FIREWALL, TC_ACT_UNSPEC);
    }

    if (is_ipv4) {
        bpf_tail_call_static(skb, &egress_prog_array, IPV4_FIREWALL_EGRESS_PROG_INDEX);
    } else {
        bpf_tail_call_static(skb, &egress_prog_array, IPV6_FIREWALL_EGRESS_PROG_INDEX);
    }
    // if (ret) {
    //     ld_bpf_log("bpf_tail_call error: %d", ret);
    // }
    return wan_tc_pipeline_continue_egress(skb, EGRESS_STAGE_FIREWALL, TC_ACT_UNSPEC);
#undef BPF_LOG_TOPIC
}

SEC("tc/ingress")
int ingress_firewall(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "<<< ingress_firewall <<<"

    bool is_ipv4;
    int ret;
    if (current_pkg_type(skb, current_l3_offset, &is_ipv4) != TC_ACT_OK) {
        return wan_tc_pipeline_continue_ingress(skb, INGRESS_STAGE_FIREWALL, TC_ACT_UNSPEC);
    }

    if (is_ipv4) {
        bpf_tail_call_static(skb, &ingress_prog_array, IPV4_FIREWALL_INGRESS_PROG_INDEX);
    } else {
        bpf_tail_call_static(skb, &ingress_prog_array, IPV6_FIREWALL_INGRESS_PROG_INDEX);
    }
    return wan_tc_pipeline_continue_ingress(skb, INGRESS_STAGE_FIREWALL, TC_ACT_UNSPEC);
#undef BPF_LOG_TOPIC
}
