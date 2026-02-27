#include <vmlinux.h>

#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#include "landscape.h"
#include "land_nat_common.h"
#include "nat/nat_maps.h"
#include "land_nat_v6.h"
#include "land_nat_v4.h"

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
#define BPF_LOG_TOPIC "nat_v4_egress <<<"
    struct packet_offset_info pkg_offset = {0};
    struct inet4_pair ip_pair = {0};
    int ret = 0;

    ret = scan_packet(skb, current_l3_offset, &pkg_offset);
    if (ret) {
        return ret;
    }

    ret = is_handle_protocol(pkg_offset.l4_protocol);
    if (ret != TC_ACT_OK) {
        return ret;
    }

    ret = read_packet_info4(skb, &pkg_offset, &ip_pair);
    if (ret) {
        return ret;
    }

    ret = is_broadcast_ip4_pair(&ip_pair);
    if (ret != TC_ACT_OK) {
        return ret;
    }

    ret = frag_info_track_v4(&pkg_offset, &ip_pair);
    if (ret != TC_ACT_OK) {
        return TC_ACT_SHOT;
    }

    bool is_icmpx_error = is_icmp_error_pkt(&pkg_offset);
    bool allow_create_mapping = !is_icmpx_error && pkt_allow_initiating_ct(pkg_offset.pkt_type);

    // Unified lookup: static and dynamic in nat4_mappings
    struct nat_mapping_value_v4 *nat_egress_value, *nat_ingress_value;

    ret = egress_lookup_or_new_mapping_v4(skb, pkg_offset.l4_protocol, allow_create_mapping,
                                          &ip_pair, &nat_egress_value, &nat_ingress_value);
    if (ret != TC_ACT_OK) {
        return TC_ACT_SHOT;
    }

    if (nat_egress_value == NULL) {
        bpf_log_info("nat_egress_value is null");
        return TC_ACT_SHOT;
    }

    // Port reuse check (skip for static and ICMP)
    if (!nat_egress_value->is_static && nat_egress_value->is_allow_reuse == 0 &&
        pkg_offset.l4_protocol != IPPROTO_ICMP) {
        if (ip_pair.dst_addr.addr != nat_egress_value->trigger_addr ||
            ip_pair.dst_port != nat_egress_value->trigger_port) {
            bpf_log_info("FLOW_ALLOW_REUSE MARK not set, DROP PACKET");
            return TC_ACT_SHOT;
        }
    }

    // Sync is_allow_reuse when dst==trigger
    if (!nat_egress_value->is_static && ip_pair.dst_addr.addr == nat_egress_value->trigger_addr &&
        ip_pair.dst_port == nat_egress_value->trigger_port) {
        bool allow_reuse = get_flow_allow_reuse_port(skb->mark);
        u8 new_val = allow_reuse ? 1 : 0;
        nat_egress_value->is_allow_reuse = new_val;
        if (nat_ingress_value) {
            nat_ingress_value->is_allow_reuse = new_val;
        }
    }

    // Determine An (NAT addr on WAN side)
    struct inet4_addr nat_addr;
    if (nat_egress_value->is_static) {
        struct wan_ip_info_key wan_search_key = {0};
        wan_search_key.ifindex = skb->ifindex;
        wan_search_key.l3_protocol = LANDSCAPE_IPV4_TYPE;

        struct wan_ip_info_value *wan_ip_info =
            bpf_map_lookup_elem(&wan_ip_binding, &wan_search_key);
        if (!wan_ip_info) {
            bpf_log_info("can't find the wan ip, using ifindex: %d", skb->ifindex);
            return TC_ACT_SHOT;
        }
        nat_addr.addr = wan_ip_info->addr.ip;
    } else {
        nat_addr.addr = nat_egress_value->addr;
    }

    // CT key: {As:Ps, An:Pn} — static NAT also creates CT
    struct inet4_pair server_nat_pair = {
        .src_addr = ip_pair.dst_addr,
        .src_port = ip_pair.dst_port,
        .dst_addr = nat_addr,
        .dst_port = nat_egress_value->port,
    };

    // For ICMP, use the allocated ID (Pn) instead of original ID (Ps) in CT key
    // so that Ingress can match the CT with the same key
    if (pkg_offset.l4_protocol == IPPROTO_ICMP) {
        server_nat_pair.src_port = nat_egress_value->port;
    }

    struct nat_timer_value_v4 *ct_value;
    ret = lookup_or_new_ct(skb, pkg_offset.l4_protocol, allow_create_mapping, &server_nat_pair,
                           &ip_pair.src_addr, ip_pair.src_port, NAT_MAPPING_EGRESS, &ct_value);
    if (ret == TIMER_NOT_FOUND || ret == TIMER_ERROR) {
        return TC_ACT_SHOT;
    }
    if (!is_icmpx_error || ct_value != NULL) {
        ct_state_transition(pkg_offset.l4_protocol, pkg_offset.pkt_type, NAT_MAPPING_EGRESS,
                            ct_value);
        nat_metric_accumulate(skb, false, ct_value);
    }

    // modify source: Ac:Pc → An:Pn
    struct nat_action_v4 action = {
        .from_addr = ip_pair.src_addr,
        .from_port = ip_pair.src_port,
        .to_addr = nat_addr,
        .to_port = nat_egress_value->port,
    };

    ret = modify_headers_v4(skb, is_icmpx_error, pkg_offset.l4_protocol, current_l3_offset,
                            pkg_offset.l4_offset, pkg_offset.icmp_error_inner_l4_offset, true,
                            &action);
    if (ret) {
        bpf_log_error("failed to update csum, err:%d", ret);
        return TC_ACT_SHOT;
    }

    return TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}

SEC("tc/ingress")
int nat_v4_ingress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "nat_v4_ingress >>>"

    struct packet_offset_info pkg_offset = {0};
    struct inet4_pair ip_pair = {0};
    int ret = 0;

    ret = scan_packet(skb, current_l3_offset, &pkg_offset);
    if (ret) {
        return ret;
    }

    ret = is_handle_protocol(pkg_offset.l4_protocol);
    if (ret != TC_ACT_OK) {
        return ret;
    }

    ret = read_packet_info4(skb, &pkg_offset, &ip_pair);
    if (ret) {
        return ret;
    }

    ret = is_broadcast_ip4_pair(&ip_pair);
    if (ret != TC_ACT_OK) {
        return ret;
    }

    ret = frag_info_track_v4(&pkg_offset, &ip_pair);
    if (ret != TC_ACT_OK) {
        return TC_ACT_SHOT;
    }

    bool is_icmpx_error = is_icmp_error_pkt(&pkg_offset);

    // Unified lookup with addr=0 fallback for static
    struct nat_mapping_value_v4 *nat_ingress_value;

    ret = ingress_lookup_or_new_mapping4(skb, pkg_offset.l4_protocol, &ip_pair, &nat_ingress_value);
    if (ret != TC_ACT_OK) {
        return TC_ACT_SHOT;
    }

    if (nat_ingress_value == NULL) {
        bpf_log_info("nat_ingress_value is null");
        return TC_ACT_SHOT;
    }

    // Port reuse check: src!=trigger && is_allow_reuse==0 → DROP (skip for static and ICMP)
    if (!nat_ingress_value->is_static && nat_ingress_value->is_allow_reuse == 0 &&
        pkg_offset.l4_protocol != IPPROTO_ICMP) {
        if (ip_pair.src_addr.addr != nat_ingress_value->trigger_addr ||
            ip_pair.src_port != nat_ingress_value->trigger_port) {
            bpf_log_info("ingress FLOW_ALLOW_REUSE not set, DROP PACKET");
            return TC_ACT_SHOT;
        }
    }

    // Static mark for routing
    if (nat_ingress_value->is_static) {
        u32 mark = skb->mark;
        barrier_var(mark);
        skb->mark = replace_cache_mask(mark, INGRESS_STATIC_MARK);
    }

    // Determine lan_ip (Ac)
    struct inet4_addr lan_ip;
    if (nat_ingress_value->is_static && nat_ingress_value->addr == 0) {
        lan_ip.addr = ip_pair.dst_addr.addr;
    } else {
        lan_ip.addr = nat_ingress_value->addr;
    }

    // CT key: {As:Ps, An:Pn} = {src, dst}
    struct inet4_pair server_nat_pair = {
        .src_addr = ip_pair.src_addr,
        .src_port = ip_pair.src_port,
        .dst_addr = ip_pair.dst_addr,
        .dst_port = ip_pair.dst_port,
    };

    // Dynamic: CT must already exist (do_new=false)
    // Static: can create CT for inbound connections
    bool do_new_ct = nat_ingress_value->is_static
                         ? (!is_icmpx_error && pkt_allow_initiating_ct(pkg_offset.pkt_type))
                         : false;

    struct nat_timer_value_v4 *ct_value;
    ret = lookup_or_new_ct(skb, pkg_offset.l4_protocol, do_new_ct, &server_nat_pair, &lan_ip,
                           nat_ingress_value->port, NAT_MAPPING_INGRESS, &ct_value);
    if (ret == TIMER_NOT_FOUND || ret == TIMER_ERROR) {
        bpf_log_info("connect ret :%u", ret);
        return TC_ACT_SHOT;
    }
    if (!is_icmpx_error || ct_value != NULL) {
        ct_state_transition(pkg_offset.l4_protocol, pkg_offset.pkt_type, NAT_MAPPING_INGRESS,
                            ct_value);
        nat_metric_accumulate(skb, true, ct_value);
    }

    // modify dest: An:Pn → Ac:Pc
    struct nat_action_v4 action = {
        .from_addr = ip_pair.dst_addr,
        .from_port = ip_pair.dst_port,
        .to_addr = lan_ip,
        .to_port = nat_ingress_value->port,
    };

    ret = modify_headers_v4(skb, is_icmpx_error, pkg_offset.l4_protocol, current_l3_offset,
                            pkg_offset.l4_offset, pkg_offset.icmp_error_inner_l4_offset, false,
                            &action);
    if (ret) {
        bpf_log_error("failed to update csum, err:%d", ret);
        return TC_ACT_SHOT;
    }

    return TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}

SEC("tc/egress")
int nat_v6_egress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "nat_v6_egress <<<"

    struct packet_offset_info pkg_offset = {0};
    struct inet_pair ip_pair = {0};
    int ret = 0;

    ret = scan_packet(skb, current_l3_offset, &pkg_offset);
    if (ret) {
        return ret;
    }

    ret = is_handle_protocol(pkg_offset.l4_protocol);
    if (ret != TC_ACT_OK) {
        return ret;
    }

    ret = read_packet_info(skb, &pkg_offset, &ip_pair);
    if (ret) {
        return ret;
    }

    ret = is_broadcast_ip_pair(pkg_offset.l3_protocol, &ip_pair);
    if (ret != TC_ACT_OK) {
        return ret;
    }

    ret = frag_info_track(&pkg_offset, &ip_pair);
    if (ret != TC_ACT_OK) {
        return TC_ACT_SHOT;
    }

    return ipv6_egress_prefix_check_and_replace(skb, &pkg_offset, &ip_pair);
#undef BPF_LOG_TOPIC
}

SEC("tc/ingress")
int nat_v6_ingress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "nat_v6_ingress >>>"

    struct packet_offset_info pkg_offset = {0};
    struct inet_pair ip_pair = {0};
    int ret = 0;

    ret = scan_packet(skb, current_l3_offset, &pkg_offset);
    if (ret) {
        return ret;
    }

    ret = is_handle_protocol(pkg_offset.l4_protocol);
    if (ret != TC_ACT_OK) {
        return ret;
    }

    ret = read_packet_info(skb, &pkg_offset, &ip_pair);
    if (ret) {
        return ret;
    }

    ret = is_broadcast_ip_pair(pkg_offset.l3_protocol, &ip_pair);
    if (ret != TC_ACT_OK) {
        return ret;
    }

    ret = frag_info_track(&pkg_offset, &ip_pair);
    if (ret != TC_ACT_OK) {
        return TC_ACT_SHOT;
    }

    return ipv6_ingress_prefix_check_and_replace(skb, &pkg_offset, &ip_pair);
#undef BPF_LOG_TOPIC
}

SEC("tc/ingress")
int ingress_nat(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC ">>> ingress_nat >>>"

    bool is_ipv4;
    int ret;

    if (likely(current_l3_offset > 0)) {
        ret = is_broadcast_mac(skb);
        if (unlikely(ret != TC_ACT_OK)) {
            return ret;
        }
    }

    ret = current_pkg_type(skb, current_l3_offset, &is_ipv4);
    if (unlikely(ret != TC_ACT_OK)) {
        return TC_ACT_UNSPEC;
    }

    if (is_ipv4) {
        bpf_tail_call_static(skb, &ingress_prog_array, IPV4_NAT_INGRESS_PROG_INDEX);
        bpf_printk("bpf_tail_call_static error");
    } else {
        bpf_tail_call_static(skb, &ingress_prog_array, IPV6_NAT_INGRESS_PROG_INDEX);
        bpf_printk("bpf_tail_call_static error");
    }

    return TC_ACT_SHOT;
#undef BPF_LOG_TOPIC
}

SEC("tc/egress")
int egress_nat(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "<<< egress_nat <<<"

    bool is_ipv4;
    int ret;

    if (likely(current_l3_offset > 0)) {
        ret = is_broadcast_mac(skb);
        if (unlikely(ret != TC_ACT_OK)) {
            return ret;
        }
    }

    ret = current_pkg_type(skb, current_l3_offset, &is_ipv4);
    if (unlikely(ret != TC_ACT_OK)) {
        return TC_ACT_UNSPEC;
    }

    if (is_ipv4) {
        bpf_tail_call_static(skb, &egress_prog_array, IPV4_NAT_EGRESS_PROG_INDEX);
        bpf_printk("bpf_tail_call_static error");
    } else {
        bpf_tail_call_static(skb, &egress_prog_array, IPV6_NAT_EGRESS_PROG_INDEX);
        bpf_printk("bpf_tail_call_static error");
    }

    return TC_ACT_SHOT;
#undef BPF_LOG_TOPIC
}
