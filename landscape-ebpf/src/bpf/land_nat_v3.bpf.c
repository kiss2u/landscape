#include <vmlinux.h>

#include <bpf/bpf_core_read.h>
#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

#include "land_nat4_v3.h"
#include "land_nat6_v3.h"
#include "landscape.h"

char LICENSE[] SEC("license") = "Dual BSD/GPL";
const volatile u8 LOG_LEVEL = BPF_LOG_LEVEL_DEBUG;

#undef BPF_LOG_LEVEL
#undef BPF_LOG_TOPIC
#define BPF_LOG_LEVEL LOG_LEVEL

#define IPV4_NAT_EGRESS_PROG_INDEX 0
#define IPV4_NAT_INGRESS_PROG_INDEX 0
#define IPV6_NAT_EGRESS_PROG_INDEX 1
#define IPV6_NAT_INGRESS_PROG_INDEX 1

const volatile u32 current_l3_offset = 14;

SEC("tc/egress") int nat_v4_egress(struct __sk_buff *skb);
SEC("tc/ingress") int nat_v4_ingress(struct __sk_buff *skb);
SEC("tc/egress") int nat_v6_egress(struct __sk_buff *skb);
SEC("tc/ingress") int nat_v6_ingress(struct __sk_buff *skb);

struct {
    __uint(type, BPF_MAP_TYPE_PROG_ARRAY);
    __uint(max_entries, 2);
    __uint(key_size, sizeof(u32));
    __uint(value_size, sizeof(u32));
    __array(values, int());
} ingress_prog_array SEC(".maps") = {
    .values =
        {
            [IPV4_NAT_INGRESS_PROG_INDEX] = (void *)&nat_v4_ingress,
            [IPV6_NAT_INGRESS_PROG_INDEX] = (void *)&nat_v6_ingress,
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
            [IPV4_NAT_EGRESS_PROG_INDEX] = (void *)&nat_v4_egress,
            [IPV6_NAT_EGRESS_PROG_INDEX] = (void *)&nat_v6_egress,
        },
};

SEC("tc/egress")
int nat_v4_egress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "nat_v4_egress_v3 <<<"
    struct packet_offset_info pkg_offset = {0};
    struct inet4_pair ip_pair = {0};
    struct nat4_lookup_result_v3 lookup = {0};
    int ret = 0;

    ret = scan_packet(skb, current_l3_offset, &pkg_offset);
    if (ret) return ret;
    ret = is_handle_protocol(pkg_offset.l4_protocol);
    if (ret != TC_ACT_OK) return ret;
    ret = read_packet_info4(skb, &pkg_offset, &ip_pair);
    if (ret) return ret;
    ret = is_broadcast_ip4_pair(&ip_pair);
    if (ret != TC_ACT_OK) return ret;
    ret = frag_info_track_v4(&pkg_offset, &ip_pair);
    if (ret != TC_ACT_OK) return TC_ACT_SHOT;

    bool is_icmpx_error = is_icmp_error_pkt(&pkg_offset);
    bool allow_create_mapping = !is_icmpx_error && pkt_allow_initiating_ct(pkg_offset.pkt_type);

    ret = nat4_v3_egress_lookup_or_new_mapping(skb, pkg_offset.l4_protocol, allow_create_mapping,
                                               &ip_pair, &lookup);
    if (ret != TC_ACT_OK || !lookup.egress) {
        return TC_ACT_SHOT;
    }

    if (!lookup.is_static && lookup.egress->is_allow_reuse == 0 &&
        pkg_offset.l4_protocol != IPPROTO_ICMP) {
        if (ip_pair.dst_addr.addr != lookup.egress->trigger_addr ||
            ip_pair.dst_port != lookup.egress->trigger_port) {
            return TC_ACT_SHOT;
        }
    }

    if (!lookup.is_static && ip_pair.dst_addr.addr == lookup.egress->trigger_addr &&
        ip_pair.dst_port == lookup.egress->trigger_port) {
        u8 allow = get_flow_allow_reuse_port(skb->mark) ? 1 : 0;
        lookup.egress->is_allow_reuse = allow;
        if (lookup.ingress) lookup.ingress->is_allow_reuse = allow;
    }

    struct inet4_addr nat_addr = {0};
    if (lookup.is_static) {
        struct wan_ip_info_key wan_search_key = {
            .ifindex = skb->ifindex,
            .l3_protocol = LANDSCAPE_IPV4_TYPE,
        };
        struct wan_ip_info_value *wan_ip_info =
            bpf_map_lookup_elem(&wan_ip_binding, &wan_search_key);
        if (!wan_ip_info) return TC_ACT_SHOT;
        nat_addr.addr = wan_ip_info->addr.ip;
    } else {
        nat_addr.addr = lookup.egress->addr;
    }

    struct inet4_pair server_nat_pair = {
        .src_addr = ip_pair.dst_addr,
        .src_port = ip_pair.dst_port,
        .dst_addr = nat_addr,
        .dst_port = lookup.egress->port,
    };
    if (pkg_offset.l4_protocol == IPPROTO_ICMP) {
        server_nat_pair.src_port = lookup.egress->port;
    }

    struct nat_timer_value_v4_v3 *ct_value = NULL;
    u16 generation = lookup.state ? lookup.state->generation : 0;
    ret = nat4_v3_lookup_or_new_ct(skb, pkg_offset.l4_protocol, allow_create_mapping,
                                   &server_nat_pair, &ip_pair.src_addr, ip_pair.src_port,
                                   NAT_MAPPING_EGRESS, generation,
                                   lookup.created || lookup.is_static, lookup.state, &ct_value);
    if (ret == TIMER_NOT_FOUND || ret == TIMER_ERROR) {
        if (lookup.created && !lookup.is_static) {
            nat4_v3_delete_mapping_and_state(pkg_offset.l4_protocol, nat_addr.addr,
                                             lookup.egress->port, ip_pair.src_addr.addr,
                                             ip_pair.src_port);
            (void)nat4_v3_queue_push(pkg_offset.l4_protocol, &lookup.alloc_item);
        }
        return TC_ACT_SHOT;
    }

    if (!is_icmpx_error || ct_value != NULL) {
        ct_state_transition(pkg_offset.l4_protocol, pkg_offset.pkt_type, NAT_MAPPING_EGRESS,
                            nat4_v3_timer_base(ct_value));
        nat_metric_accumulate(skb, false, nat4_v3_timer_base(ct_value));
    }

    struct nat_action_v4 action = {
        .from_addr = ip_pair.src_addr,
        .from_port = ip_pair.src_port,
        .to_addr = nat_addr,
        .to_port = lookup.egress->port,
    };

    ret = modify_headers_v4(skb, is_icmpx_error, pkg_offset.l4_protocol, current_l3_offset,
                            pkg_offset.l4_offset, pkg_offset.icmp_error_inner_l4_offset, true,
                            &action);
    return ret ? TC_ACT_SHOT : TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}

SEC("tc/ingress")
int nat_v4_ingress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "nat_v4_ingress_v3 >>>"
    struct packet_offset_info pkg_offset = {0};
    struct inet4_pair ip_pair = {0};
    struct nat_mapping_value_v4 *nat_ingress_value = NULL;
    struct nat4_mapping_state_v3 *state = NULL;
    int ret = 0;

    ret = scan_packet(skb, current_l3_offset, &pkg_offset);
    if (ret) return ret;
    ret = is_handle_protocol(pkg_offset.l4_protocol);
    if (ret != TC_ACT_OK) return ret;
    ret = read_packet_info4(skb, &pkg_offset, &ip_pair);
    if (ret) return ret;
    ret = is_broadcast_ip4_pair(&ip_pair);
    if (ret != TC_ACT_OK) return ret;
    ret = frag_info_track_v4(&pkg_offset, &ip_pair);
    if (ret != TC_ACT_OK) return TC_ACT_SHOT;

    bool is_icmpx_error = is_icmp_error_pkt(&pkg_offset);

    ret = nat4_v3_ingress_lookup_mapping(pkg_offset.l4_protocol, &ip_pair, &nat_ingress_value,
                                         &state);
    if (ret != TC_ACT_OK || !nat_ingress_value) {
        return TC_ACT_SHOT;
    }

    if (!nat_ingress_value->is_static && nat_ingress_value->is_allow_reuse == 0 &&
        pkg_offset.l4_protocol != IPPROTO_ICMP) {
        if (ip_pair.src_addr.addr != nat_ingress_value->trigger_addr ||
            ip_pair.src_port != nat_ingress_value->trigger_port) {
            return TC_ACT_SHOT;
        }
    }

    if (nat_ingress_value->is_static) {
        u32 mark = skb->mark;
        barrier_var(mark);
        skb->mark = replace_cache_mask(mark, INGRESS_STATIC_MARK);
    }

    struct inet4_addr lan_ip = {0};
    if (nat_ingress_value->is_static && nat_ingress_value->addr == 0) {
        lan_ip.addr = ip_pair.dst_addr.addr;
    } else {
        lan_ip.addr = nat_ingress_value->addr;
    }

    struct inet4_pair server_nat_pair = {
        .src_addr = ip_pair.src_addr,
        .src_port = ip_pair.src_port,
        .dst_addr = ip_pair.dst_addr,
        .dst_port = ip_pair.dst_port,
    };

    bool do_new_ct = nat_ingress_value->is_static
                         ? (!is_icmpx_error && pkt_allow_initiating_ct(pkg_offset.pkt_type))
                         : (nat_ingress_value->is_allow_reuse && !is_icmpx_error &&
                            pkt_allow_initiating_ct(pkg_offset.pkt_type));

    struct nat_timer_value_v4_v3 *ct_value = NULL;
    u16 generation = state ? state->generation : 0;
    ret = nat4_v3_lookup_or_new_ct(skb, pkg_offset.l4_protocol, do_new_ct, &server_nat_pair,
                                   &lan_ip, nat_ingress_value->port, NAT_MAPPING_INGRESS,
                                   generation, nat_ingress_value->is_static, state, &ct_value);
    if (ret == TIMER_NOT_FOUND || ret == TIMER_ERROR) {
        return TC_ACT_SHOT;
    }

    if (!is_icmpx_error || ct_value != NULL) {
        ct_state_transition(pkg_offset.l4_protocol, pkg_offset.pkt_type, NAT_MAPPING_INGRESS,
                            nat4_v3_timer_base(ct_value));
        nat_metric_accumulate(skb, true, nat4_v3_timer_base(ct_value));
    }

    struct nat_action_v4 action = {
        .from_addr = ip_pair.dst_addr,
        .from_port = ip_pair.dst_port,
        .to_addr = lan_ip,
        .to_port = nat_ingress_value->port,
    };

    ret = modify_headers_v4(skb, is_icmpx_error, pkg_offset.l4_protocol, current_l3_offset,
                            pkg_offset.l4_offset, pkg_offset.icmp_error_inner_l4_offset, false,
                            &action);
    return ret ? TC_ACT_SHOT : TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}

SEC("tc/egress")
int nat_v6_egress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "nat_v6_egress_v3 <<<"
    struct packet_offset_info pkg_offset = {0};
    struct inet_pair ip_pair = {0};
    int ret = 0;

    ret = scan_packet(skb, current_l3_offset, &pkg_offset);
    if (ret) return ret;
    ret = is_handle_protocol(pkg_offset.l4_protocol);
    if (ret != TC_ACT_OK) return ret;
    ret = read_packet_info(skb, &pkg_offset, &ip_pair);
    if (ret) return ret;
    ret = is_broadcast_ip_pair(pkg_offset.l3_protocol, &ip_pair);
    if (ret != TC_ACT_OK) return ret;
    ret = frag_info_track(&pkg_offset, &ip_pair);
    if (ret != TC_ACT_OK) return TC_ACT_SHOT;
    return ipv6_egress_prefix_check_and_replace(skb, &pkg_offset, &ip_pair);
#undef BPF_LOG_TOPIC
}

SEC("tc/ingress")
int nat_v6_ingress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "nat_v6_ingress_v3 >>>"
    struct packet_offset_info pkg_offset = {0};
    struct inet_pair ip_pair = {0};
    int ret = 0;

    ret = scan_packet(skb, current_l3_offset, &pkg_offset);
    if (ret) return ret;
    ret = is_handle_protocol(pkg_offset.l4_protocol);
    if (ret != TC_ACT_OK) return ret;
    ret = read_packet_info(skb, &pkg_offset, &ip_pair);
    if (ret) return ret;
    ret = is_broadcast_ip_pair(pkg_offset.l3_protocol, &ip_pair);
    if (ret != TC_ACT_OK) return ret;
    ret = frag_info_track(&pkg_offset, &ip_pair);
    if (ret != TC_ACT_OK) return TC_ACT_SHOT;
    return ipv6_ingress_prefix_check_and_replace(skb, &pkg_offset, &ip_pair);
#undef BPF_LOG_TOPIC
}

SEC("tc/ingress")
int ingress_nat(struct __sk_buff *skb) {
    bool is_ipv4;
    int ret;

    if (likely(current_l3_offset > 0)) {
        ret = is_broadcast_mac(skb);
        if (unlikely(ret != TC_ACT_OK)) return ret;
    }

    ret = current_pkg_type(skb, current_l3_offset, &is_ipv4);
    if (unlikely(ret != TC_ACT_OK)) return TC_ACT_UNSPEC;

    if (is_ipv4) {
        bpf_tail_call_static(skb, &ingress_prog_array, IPV4_NAT_INGRESS_PROG_INDEX);
    } else {
        bpf_tail_call_static(skb, &ingress_prog_array, IPV6_NAT_INGRESS_PROG_INDEX);
    }

    return TC_ACT_SHOT;
}

SEC("tc/egress")
int egress_nat(struct __sk_buff *skb) {
    bool is_ipv4;
    int ret;

    if (likely(current_l3_offset > 0)) {
        ret = is_broadcast_mac(skb);
        if (unlikely(ret != TC_ACT_OK)) return ret;
    }

    ret = current_pkg_type(skb, current_l3_offset, &is_ipv4);
    if (unlikely(ret != TC_ACT_OK)) return TC_ACT_UNSPEC;

    if (is_ipv4) {
        bpf_tail_call_static(skb, &egress_prog_array, IPV4_NAT_EGRESS_PROG_INDEX);
    } else {
        bpf_tail_call_static(skb, &egress_prog_array, IPV6_NAT_EGRESS_PROG_INDEX);
    }

    return TC_ACT_SHOT;
}
