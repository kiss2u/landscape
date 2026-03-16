#ifndef LD_NAT4_V3_H
#define LD_NAT4_V3_H

#include <vmlinux.h>

#include "landscape_log.h"
#include "land_nat_common.h"
#include "nat/nat_maps.h"
#include "land_wan_ip.h"
#include "nat/nat_v3_maps.h"

volatile const u16 tcp_range_start = 32768;
volatile const u16 tcp_range_end = 65535;

volatile const u16 udp_range_start = 32768;
volatile const u16 udp_range_end = 65535;

volatile const u16 icmp_range_start = 32768;
volatile const u16 icmp_range_end = 65535;

static __always_inline int icmpx_err_l3_offset(int l4_off) {
    return l4_off + sizeof(struct icmphdr);
}

#define L3_CSUM_REPLACE_OR_SHOT(skb_ptr, csum_offset, old_val, new_val, size)                      \
    do {                                                                                           \
        int _ret = bpf_l3_csum_replace(skb_ptr, csum_offset, old_val, new_val, size);              \
        if (_ret) {                                                                                \
            bpf_printk("l3_csum_replace err: %d", _ret);                                           \
            return TC_ACT_SHOT;                                                                    \
        }                                                                                          \
    } while (0)

#define L4_CSUM_REPLACE_OR_SHOT(skb_ptr, csum_offset, old_val, new_val, len_plus_flags)            \
    do {                                                                                           \
        int _ret = bpf_l4_csum_replace(skb_ptr, csum_offset, old_val, new_val, len_plus_flags);    \
        if (_ret) {                                                                                \
            bpf_printk("l4_csum_replace err: %d", _ret);                                           \
            return TC_ACT_SHOT;                                                                    \
        }                                                                                          \
    } while (0)

static __always_inline int ipv4_update_csum_inner_macro(struct __sk_buff *skb, u32 l4_csum_off,
                                                        __be32 from_addr, __be16 from_port,
                                                        __be32 to_addr, __be16 to_port,
                                                        bool l4_pseudo, bool l4_mangled_0) {
    u16 csum;
    if (l4_mangled_0) {
        READ_SKB_U16(skb, l4_csum_off, csum);
    }

    if (!l4_mangled_0 || csum != 0) {
        L3_CSUM_REPLACE_OR_SHOT(skb, l4_csum_off, from_port, to_port, 2);

        if (l4_pseudo) {
            L3_CSUM_REPLACE_OR_SHOT(skb, l4_csum_off, from_addr, to_addr, 4);
        }
    }
}

static __always_inline int ipv4_update_csum_icmp_err_macro(struct __sk_buff *skb, u32 icmp_csum_off,
                                                           u32 err_ip_check_off,
                                                           u32 err_l4_csum_off, __be32 from_addr,
                                                           __be16 from_port, __be32 to_addr,
                                                           __be16 to_port, bool err_l4_pseudo,
                                                           bool l4_mangled_0) {
    u16 prev_csum;
    u16 curr_csum;
    u16 *tmp_ptr;

    if (VALIDATE_READ_DATA(skb, &tmp_ptr, err_ip_check_off, sizeof(*tmp_ptr))) {
        return 1;
    }
    prev_csum = *tmp_ptr;

    L3_CSUM_REPLACE_OR_SHOT(skb, err_ip_check_off, from_addr, to_addr, 4);

    if (VALIDATE_READ_DATA(skb, &tmp_ptr, err_ip_check_off, sizeof(*tmp_ptr))) {
        return 1;
    }
    curr_csum = *tmp_ptr;
    L4_CSUM_REPLACE_OR_SHOT(skb, icmp_csum_off, prev_csum, curr_csum, 2);

    if (VALIDATE_READ_DATA(skb, &tmp_ptr, err_l4_csum_off, sizeof(*tmp_ptr)) == 0) {
        prev_csum = *tmp_ptr;
        ipv4_update_csum_inner_macro(skb, err_l4_csum_off, from_addr, from_port, to_addr, to_port,
                                     err_l4_pseudo, l4_mangled_0);

        if (VALIDATE_READ_DATA(skb, &tmp_ptr, err_l4_csum_off, sizeof(*tmp_ptr))) {
            return 1;
        }
        curr_csum = *tmp_ptr;
        L4_CSUM_REPLACE_OR_SHOT(skb, icmp_csum_off, prev_csum, curr_csum, 2);
    }

    L4_CSUM_REPLACE_OR_SHOT(skb, icmp_csum_off, from_addr, to_addr, 4);
    L4_CSUM_REPLACE_OR_SHOT(skb, icmp_csum_off, from_port, to_port, 2);

    return 0;
}

static __always_inline int modify_headers_v4(struct __sk_buff *skb, bool is_icmpx_error, u8 nexthdr,
                                             u32 current_l3_offset, int l4_off, int err_l4_off,
                                             bool is_modify_source,
                                             const struct nat_action_v4 *action) {
#define BPF_LOG_TOPIC "modify_headers_v4"
    int ret;
    int l4_to_port_off;
    int l4_to_check_off;
    bool l4_check_pseudo;
    bool l4_check_mangle_0;

    int ip_offset =
        is_modify_source ? offsetof(struct iphdr, saddr) : offsetof(struct iphdr, daddr);

    ret = bpf_skb_store_bytes(skb, current_l3_offset + ip_offset, &action->to_addr.addr,
                              sizeof(action->to_addr.addr), 0);
    if (ret) return ret;

    L3_CSUM_REPLACE_OR_SHOT(skb, current_l3_offset + offsetof(struct iphdr, check),
                            action->from_addr.addr, action->to_addr.addr, 4);

    if (l4_off == 0) return 0;

    switch (nexthdr) {
    case IPPROTO_TCP:
        l4_to_port_off =
            is_modify_source ? offsetof(struct tcphdr, source) : offsetof(struct tcphdr, dest);
        l4_to_check_off = offsetof(struct tcphdr, check);
        l4_check_pseudo = true;
        l4_check_mangle_0 = false;
        break;
    case IPPROTO_UDP:
        l4_to_port_off =
            is_modify_source ? offsetof(struct udphdr, source) : offsetof(struct udphdr, dest);
        l4_to_check_off = offsetof(struct udphdr, check);
        l4_check_pseudo = true;
        l4_check_mangle_0 = true;
        break;
    case IPPROTO_ICMP:
        l4_to_port_off = offsetof(struct icmphdr, un.echo.id);
        l4_to_check_off = offsetof(struct icmphdr, checksum);
        l4_check_pseudo = false;
        l4_check_mangle_0 = false;
        break;
    default:
        return 1;
    }

    if (is_icmpx_error) {
        if (nexthdr == IPPROTO_TCP || nexthdr == IPPROTO_UDP) {
            l4_to_port_off =
                is_modify_source ? offsetof(struct tcphdr, dest) : offsetof(struct tcphdr, source);
        }

        int icmpx_error_offset =
            is_modify_source ? offsetof(struct iphdr, daddr) : offsetof(struct iphdr, saddr);

        ret = bpf_skb_store_bytes(skb, icmpx_err_l3_offset(l4_off) + icmpx_error_offset,
                                  &action->to_addr.addr, sizeof(action->to_addr.addr), 0);
        if (ret) return ret;

        ret = bpf_write_port(skb, err_l4_off + l4_to_port_off, action->to_port);
        if (ret) return ret;

        if (ipv4_update_csum_icmp_err_macro(
                skb, l4_off + offsetof(struct icmphdr, checksum),
                icmpx_err_l3_offset(l4_off) + offsetof(struct iphdr, check),
                err_l4_off + l4_to_check_off, action->from_addr.addr, action->from_port,
                action->to_addr.addr, action->to_port, l4_check_pseudo, l4_check_mangle_0))
            return TC_ACT_SHOT;

    } else {
        ret = bpf_write_port(skb, l4_off + l4_to_port_off, action->to_port);
        if (ret) return ret;

        u32 l4_csum_off = l4_off + l4_to_check_off;
        u32 flags_mangled = l4_check_mangle_0 ? BPF_F_MARK_MANGLED_0 : 0;

        L4_CSUM_REPLACE_OR_SHOT(skb, l4_csum_off, action->from_port, action->to_port,
                                2 | flags_mangled);

        if (l4_check_pseudo) {
            L4_CSUM_REPLACE_OR_SHOT(skb, l4_csum_off, action->from_addr.addr, action->to_addr.addr,
                                    4 | BPF_F_PSEUDO_HDR | flags_mangled);
        }
    }

    return 0;
#undef BPF_LOG_TOPIC
}

static __always_inline void nat_metric_accumulate(struct __sk_buff *skb, bool ingress,
                                                  struct nat_timer_value_v4 *value) {
    u64 bytes = skb->len;
    if (ingress) {
        __sync_fetch_and_add(&value->ingress_bytes, bytes);
        __sync_fetch_and_add(&value->ingress_packets, 1);
    } else {
        __sync_fetch_and_add(&value->egress_bytes, bytes);
        __sync_fetch_and_add(&value->egress_packets, 1);
    }
}

static __always_inline int nat_metric_try_report_v4(struct nat_timer_key_v4 *timer_key,
                                                    struct nat_timer_value_v4 *timer_value,
                                                    u8 status) {
#define BPF_LOG_TOPIC "nat_metric_try_report_v4"

    struct nat_conn_metric_event *event;
    event = bpf_ringbuf_reserve(&nat_conn_metric_events, sizeof(struct nat_conn_metric_event), 0);
    if (event == NULL) {
        return -1;
    }

    event->src_addr.ip = timer_value->client_addr.addr;
    event->dst_addr.ip = timer_key->pair_ip.src_addr.addr;
    event->src_port = timer_value->client_port;
    event->dst_port = timer_key->pair_ip.src_port;
    event->l4_proto = timer_key->l4proto;
    event->l3_proto = LANDSCAPE_IPV4_TYPE;
    event->flow_id = timer_value->flow_id;
    event->trace_id = 0;
    event->time = bpf_ktime_get_tai_ns();
    event->create_time = timer_value->create_time;
    event->ingress_bytes = timer_value->ingress_bytes;
    event->ingress_packets = timer_value->ingress_packets;
    event->egress_bytes = timer_value->egress_bytes;
    event->egress_packets = timer_value->egress_packets;
    event->cpu_id = timer_value->cpu_id;
    event->ifindex = timer_value->ifindex;
    event->status = status;
    event->gress = timer_value->gress;
    bpf_ringbuf_submit(event, 0);

    return 0;
#undef BPF_LOG_TOPIC
}

static __always_inline bool ct_change_state(u64 *status_in_value, u64 curr_state, u64 next_state) {
    return __sync_bool_compare_and_swap(status_in_value, curr_state, next_state);
}

static __always_inline int ct_state_transition(u8 pkt_type, u8 gress,
                                               struct nat_timer_value_v4 *ct_timer_value) {
#define BPF_LOG_TOPIC "ct_state_transition"
    u64 curr_state, *modify_status = NULL;
    if (gress == NAT_MAPPING_INGRESS) {
        curr_state = ct_timer_value->server_status;
        modify_status = &ct_timer_value->server_status;
    } else {
        curr_state = ct_timer_value->client_status;
        modify_status = &ct_timer_value->client_status;
    }

#define NEW_STATE(__state)                                                                         \
    if (!ct_change_state(modify_status, curr_state, (__state))) {                                  \
        return TC_ACT_SHOT;                                                                        \
    }

    if (pkt_type == PKT_CONNLESS_V2) {
        NEW_STATE(CT_LESS_EST);
    }

    if (pkt_type == PKT_TCP_RST_V2) {
        NEW_STATE(CT_INIT);
    }

    if (pkt_type == PKT_TCP_SYN_V2) {
        NEW_STATE(CT_SYN);
    }

    if (pkt_type == PKT_TCP_FIN_V2) {
        NEW_STATE(CT_FIN);
    }

    u64 prev_state = __sync_lock_test_and_set(&ct_timer_value->status, TIMER_ACTIVE);
    if (prev_state != TIMER_ACTIVE) {
        if (ct_timer_value->client_port == TEST_PORT) {
            bpf_log_info("flush status to TIMER_ACTIVE: 20");
        }
        bpf_timer_start(&ct_timer_value->timer, REPORT_INTERVAL, 0);
    }

    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}

#define NAT4_V3_STATE_SHIFT 56
#define NAT4_V3_REF_MASK ((1ULL << NAT4_V3_STATE_SHIFT) - 1)
#define NAT4_V3_STATE_ACTIVE 1
#define NAT4_V3_STATE_CLOSED 2
#define TIMER_RELEASE_PENDING_QUEUE 41ULL
#define NAT4_V3_TIMER_STEP_DELETE_CT 1U
#define NAT4_V3_TIMER_STEP_RESTART 2U

struct nat4_lookup_result_v3 {
    struct nat_mapping_value_v4 *egress;
    struct nat_mapping_value_v4 *ingress;
    struct nat4_mapping_state_v3 *state;
    struct nat4_port_queue_value_v3 alloc_item;
    bool is_static;
    bool created;
};

static __always_inline u64 nat4_v3_state_make(u8 state, u64 refcnt) {
    return ((u64)state << NAT4_V3_STATE_SHIFT) | (refcnt & NAT4_V3_REF_MASK);
}

static __always_inline u8 nat4_v3_state_get(u64 state_ref) {
    return (u8)(state_ref >> NAT4_V3_STATE_SHIFT);
}

static __always_inline u64 nat4_v3_ref_get(u64 state_ref) { return state_ref & NAT4_V3_REF_MASK; }

static __always_inline int nat4_v3_state_try_inc(struct nat4_mapping_state_v3 *state) {
    u64 old = state->state_ref;

#pragma unroll
    for (int i = 0; i < 8; i++) {
        if (nat4_v3_state_get(old) != NAT4_V3_STATE_ACTIVE) {
            return -1;
        }
        u64 ref = nat4_v3_ref_get(old);
        if (ref == NAT4_V3_REF_MASK) {
            return -1;
        }
        u64 new_val = nat4_v3_state_make(NAT4_V3_STATE_ACTIVE, ref + 1);
        u64 prev = __sync_val_compare_and_swap(&state->state_ref, old, new_val);
        if (prev == old) {
            return 0;
        }
        old = prev;
    }

    return -1;
}

static __always_inline int nat4_v3_state_try_dec(struct nat4_mapping_state_v3 *state) {
    u64 old = state->state_ref;

#pragma unroll
    for (int i = 0; i < 8; i++) {
        if (nat4_v3_state_get(old) != NAT4_V3_STATE_ACTIVE) {
            return -1;
        }
        u64 ref = nat4_v3_ref_get(old);
        if (ref <= 1) {
            return -1;
        }
        u64 new_val = nat4_v3_state_make(NAT4_V3_STATE_ACTIVE, ref - 1);
        u64 prev = __sync_val_compare_and_swap(&state->state_ref, old, new_val);
        if (prev == old) {
            return 0;
        }
        old = prev;
    }

    return -1;
}

static __always_inline int nat4_v3_state_try_close_last(struct nat4_mapping_state_v3 *state) {
    u64 old = nat4_v3_state_make(NAT4_V3_STATE_ACTIVE, 1);
    u64 new_val = nat4_v3_state_make(NAT4_V3_STATE_CLOSED, 1);
    u64 prev = __sync_val_compare_and_swap(&state->state_ref, old, new_val);
    return prev == old ? 0 : -1;
}

static __always_inline void *nat4_v3_free_port_queue(u8 l4proto) {
    if (l4proto == IPPROTO_TCP) {
        return &nat4_tcp_free_ports_v3;
    }
    if (l4proto == IPPROTO_UDP) {
        return &nat4_udp_free_ports_v3;
    }
    return &nat4_icmp_free_ports_v3;
}

static __always_inline int nat4_v3_queue_pop(u8 l4proto, struct nat4_port_queue_value_v3 *value) {
    void *queue = nat4_v3_free_port_queue(l4proto);
    return bpf_map_pop_elem(queue, value);
}

static __always_inline int nat4_v3_queue_push(u8 l4proto,
                                              const struct nat4_port_queue_value_v3 *value) {
    void *queue = nat4_v3_free_port_queue(l4proto);
    return bpf_map_push_elem(queue, value, BPF_EXIST);
}

static __always_inline struct nat_mapping_value_v4 *
nat4_v3_lookup_static_egress(u8 l4proto, __be16 from_port, __be32 from_addr) {
    struct nat_mapping_key_v4 egress_key = {
        .gress = NAT_MAPPING_EGRESS,
        .l4proto = l4proto,
        .from_port = from_port,
        .from_addr = from_addr,
    };

    struct nat_mapping_value_v4 *value = bpf_map_lookup_elem(&nat4_st_map, &egress_key);
    if (!value && from_addr != 0) {
        egress_key.from_addr = 0;
        value = bpf_map_lookup_elem(&nat4_st_map, &egress_key);
    }
    return value;
}

static __always_inline struct nat_mapping_value_v4 *
nat4_v3_lookup_static_ingress(u8 l4proto, __be16 from_port) {
    struct nat_mapping_key_v4 ingress_key = {
        .gress = NAT_MAPPING_INGRESS,
        .l4proto = l4proto,
        .from_addr = 0,
        .from_port = from_port,
    };
    return bpf_map_lookup_elem(&nat4_st_map, &ingress_key);
}

static __always_inline bool nat4_v3_static_port_reserved(u8 l4proto, __be16 nat_port) {
    return nat4_v3_lookup_static_ingress(l4proto, nat_port) != NULL;
}

struct nat4_alloc_ctx_v3 {
    u8 l4proto;
    struct nat4_port_queue_value_v3 value;
    bool found;
};

static int nat4_v3_alloc_port_callback(u32 index, struct nat4_alloc_ctx_v3 *ctx) {
    if (nat4_v3_queue_pop(ctx->l4proto, &ctx->value) != 0) {
        return BPF_LOOP_RET_BREAK;
    }
    if (!nat4_v3_static_port_reserved(ctx->l4proto, ctx->value.port)) {
        ctx->found = true;
        return BPF_LOOP_RET_BREAK;
    }
    (void)nat4_v3_queue_push(ctx->l4proto, &ctx->value);
    return BPF_LOOP_RET_CONTINUE;
}

static __always_inline int nat4_v3_alloc_port(u8 l4proto, struct nat4_port_queue_value_v3 *out) {
    struct nat4_alloc_ctx_v3 ctx = {
        .l4proto = l4proto,
    };
    int ret = bpf_loop(NAT4_V3_PORT_QUEUE_SIZE, nat4_v3_alloc_port_callback, &ctx, 0);
    if (ret < 0 || !ctx.found) {
        return -1;
    }
    *out = ctx.value;
    return 0;
}

static __always_inline int nat4_v3_insert_mappings(const struct nat_mapping_key_v4 *egress_key,
                                                   const struct nat_mapping_value_v4 *egress_val,
                                                   struct nat_mapping_value_v4 **egress_out,
                                                   struct nat_mapping_value_v4 **ingress_out) {
    struct nat_mapping_key_v4 ingress_key = {
        .gress = NAT_MAPPING_INGRESS,
        .l4proto = egress_key->l4proto,
        .from_addr = egress_val->addr,
        .from_port = egress_val->port,
    };

    struct nat_mapping_value_v4 ingress_val = {
        .addr = egress_key->from_addr,
        .port = egress_key->from_port,
        .trigger_addr = egress_val->trigger_addr,
        .trigger_port = egress_val->trigger_port,
        .is_static = 0,
        .is_allow_reuse = egress_val->is_allow_reuse,
        .active_time = egress_val->active_time,
    };

    if (bpf_map_update_elem(&nat4_dyn_map, egress_key, egress_val, BPF_NOEXIST) != 0) {
        return -1;
    }
    if (bpf_map_update_elem(&nat4_dyn_map, &ingress_key, &ingress_val, BPF_NOEXIST) != 0) {
        bpf_map_delete_elem(&nat4_dyn_map, egress_key);
        return -1;
    }

    *egress_out = bpf_map_lookup_elem(&nat4_dyn_map, egress_key);
    *ingress_out = bpf_map_lookup_elem(&nat4_dyn_map, &ingress_key);
    if (!*egress_out || !*ingress_out) {
        bpf_map_delete_elem(&nat4_dyn_map, egress_key);
        bpf_map_delete_elem(&nat4_dyn_map, &ingress_key);
        return -1;
    }

    return 0;
}

static __always_inline struct nat4_mapping_state_v3 *
nat4_v3_lookup_state(u8 l4proto, __be32 nat_addr, __be16 nat_port) {
    struct nat_mapping_key_v4 ingress_key = {
        .gress = NAT_MAPPING_INGRESS,
        .l4proto = l4proto,
        .from_addr = nat_addr,
        .from_port = nat_port,
    };

    return bpf_map_lookup_elem(&nat4_dynamic_state_v3, &ingress_key);
}

static __always_inline int nat4_v3_insert_state(u8 l4proto, __be32 nat_addr, __be16 nat_port,
                                                u16 generation) {
    struct nat_mapping_key_v4 ingress_key = {
        .gress = NAT_MAPPING_INGRESS,
        .l4proto = l4proto,
        .from_addr = nat_addr,
        .from_port = nat_port,
    };
    struct nat4_mapping_state_v3 state = {
        .state_ref = nat4_v3_state_make(NAT4_V3_STATE_ACTIVE, 1),
        .generation = generation,
    };
    return bpf_map_update_elem(&nat4_dynamic_state_v3, &ingress_key, &state, BPF_NOEXIST);
}

static __always_inline void nat4_v3_delete_mapping_and_state(u8 l4proto, __be32 nat_addr,
                                                             __be16 nat_port, __be32 client_addr,
                                                             __be16 client_port) {
    struct nat_mapping_key_v4 ingress_key = {
        .gress = NAT_MAPPING_INGRESS,
        .l4proto = l4proto,
        .from_addr = nat_addr,
        .from_port = nat_port,
    };
    struct nat_mapping_key_v4 egress_key = {
        .gress = NAT_MAPPING_EGRESS,
        .l4proto = l4proto,
        .from_addr = client_addr,
        .from_port = client_port,
    };

    bpf_map_delete_elem(&nat4_dyn_map, &ingress_key);
    bpf_map_delete_elem(&nat4_dyn_map, &egress_key);
    bpf_map_delete_elem(&nat4_dynamic_state_v3, &ingress_key);
}

static __always_inline struct nat_timer_value_v4 *
nat4_v3_timer_base(struct nat_timer_value_v4_v3 *value) {
    return (struct nat_timer_value_v4 *)value;
}

static __always_inline u32 nat4_v3_handle_timer_step(struct nat_timer_key_v4 *key,
                                                     struct nat_timer_value_v4_v3 *value,
                                                     bool force_queue_push_fail,
                                                     int *queue_push_ret, u64 *next_timeout) {
    struct nat_timer_value_v4 *base = nat4_v3_timer_base(value);
    u64 current_status = base->status;
    u64 next_status = current_status;
    int ret;

    *queue_push_ret = -2;
    *next_timeout = REPORT_INTERVAL;

    if (current_status == TIMER_RELEASE_PENDING_QUEUE) {
        struct nat4_port_queue_value_v3 free_item = {
            .port = key->pair_ip.dst_port,
            .last_generation = value->generation_snapshot,
        };
        *queue_push_ret = force_queue_push_fail ? -1 : nat4_v3_queue_push(key->l4proto, &free_item);
        if (*queue_push_ret == 0) {
            bpf_map_delete_elem(&nat4_mapping_timer_v3, key);
            return NAT4_V3_TIMER_STEP_DELETE_CT;
        }
        *next_timeout = REPORT_INTERVAL;
        return NAT4_V3_TIMER_STEP_RESTART;
    }

    if (current_status == TIMER_RELEASE) {
        ret = nat_metric_try_report_v4(key, base, NAT_CONN_DELETE);
        if (ret) {
            *next_timeout = REPORT_INTERVAL;
            return NAT4_V3_TIMER_STEP_RESTART;
        }

        struct nat4_mapping_state_v3 *state =
            nat4_v3_lookup_state(key->l4proto, key->pair_ip.dst_addr.addr, key->pair_ip.dst_port);
        if (!state || state->generation != value->generation_snapshot) {
            bpf_map_delete_elem(&nat4_mapping_timer_v3, key);
            return NAT4_V3_TIMER_STEP_DELETE_CT;
        }

        if (value->is_final_releaser || nat4_v3_state_try_close_last(state) == 0) {
            struct nat4_port_queue_value_v3 free_item = {
                .port = key->pair_ip.dst_port,
                .last_generation = value->generation_snapshot,
            };
            nat4_v3_delete_mapping_and_state(key->l4proto, key->pair_ip.dst_addr.addr,
                                             key->pair_ip.dst_port, value->client_addr.addr,
                                             value->client_port);
            *queue_push_ret =
                force_queue_push_fail ? -1 : nat4_v3_queue_push(key->l4proto, &free_item);
            if (*queue_push_ret == 0) {
                bpf_map_delete_elem(&nat4_mapping_timer_v3, key);
                return NAT4_V3_TIMER_STEP_DELETE_CT;
            }
            value->status = TIMER_RELEASE_PENDING_QUEUE;
            *next_timeout = REPORT_INTERVAL;
            return NAT4_V3_TIMER_STEP_RESTART;
        }

        if (nat4_v3_state_try_dec(state) == 0) {
            bpf_map_delete_elem(&nat4_mapping_timer_v3, key);
            return NAT4_V3_TIMER_STEP_DELETE_CT;
        }

        bpf_map_delete_elem(&nat4_mapping_timer_v3, key);
        return NAT4_V3_TIMER_STEP_DELETE_CT;
    }

    ret = nat_metric_try_report_v4(key, base, NAT_CONN_ACTIVE);
    if (ret) {
        *next_timeout = REPORT_INTERVAL;
        return NAT4_V3_TIMER_STEP_RESTART;
    }

    if (current_status == TIMER_ACTIVE) {
        next_status = TIMER_TIMEOUT_1;
        *next_timeout = REPORT_INTERVAL;
    } else if (current_status == TIMER_TIMEOUT_1) {
        next_status = TIMER_TIMEOUT_2;
        *next_timeout = REPORT_INTERVAL;
    } else if (current_status == TIMER_TIMEOUT_2) {
        struct nat4_mapping_state_v3 *state =
            nat4_v3_lookup_state(key->l4proto, key->pair_ip.dst_addr.addr, key->pair_ip.dst_port);
        value->is_final_releaser = state && state->generation == value->generation_snapshot &&
                                           nat4_v3_state_try_close_last(state) == 0
                                       ? 1
                                       : 0;
        next_status = TIMER_RELEASE;
        if (key->l4proto == IPPROTO_TCP) {
            if (value->client_status == CT_SYN && value->server_status == CT_SYN) {
                *next_timeout = TCP_TIMEOUT;
            } else {
                *next_timeout = TCP_SYN_TIMEOUT;
            }
        } else {
            *next_timeout = UDP_TIMEOUT;
        }
    } else {
        next_status = TIMER_TIMEOUT_2;
        *next_timeout = REPORT_INTERVAL;
    }

    if (__sync_val_compare_and_swap(&value->status, current_status, next_status) !=
        current_status) {
        *next_timeout = REPORT_INTERVAL;
        return NAT4_V3_TIMER_STEP_RESTART;
    }

    return NAT4_V3_TIMER_STEP_RESTART;
}

static int timer_clean_callback_v3(void *map_, struct nat_timer_key_v4 *key,
                                   struct nat_timer_value_v4_v3 *value) {
#define BPF_LOG_TOPIC "timer_clean_callback_v3"
    int queue_push_ret = -2;
    u64 next_timeout = REPORT_INTERVAL;
    u32 action = nat4_v3_handle_timer_step(key, value, false, &queue_push_ret, &next_timeout);

    if (action == NAT4_V3_TIMER_STEP_RESTART) {
        bpf_timer_start(&value->timer, next_timeout, 0);
    }
    return 0;
#undef BPF_LOG_TOPIC
}

static __always_inline struct nat_timer_value_v4_v3 *
nat4_v3_insert_ct(const struct nat_timer_key_v4 *key, const struct nat_timer_value_v4_v3 *val) {
    if (bpf_map_update_elem(&nat4_mapping_timer_v3, key, val, BPF_NOEXIST) != 0) {
        return NULL;
    }
    struct nat_timer_value_v4_v3 *value = bpf_map_lookup_elem(&nat4_mapping_timer_v3, key);
    if (!value) {
        return NULL;
    }
    if (bpf_timer_init(&value->timer, &nat4_mapping_timer_v3, CLOCK_MONOTONIC) != 0) {
        goto err;
    }
    if (bpf_timer_set_callback(&value->timer, timer_clean_callback_v3) != 0) {
        goto err;
    }
    if (bpf_timer_start(&value->timer, REPORT_INTERVAL, 0) != 0) {
        goto err;
    }
    return value;
err:
    bpf_map_delete_elem(&nat4_mapping_timer_v3, key);
    return NULL;
}

static __always_inline int nat4_v3_lookup_or_new_ct(struct __sk_buff *skb, u8 l4proto, bool do_new,
                                                    const struct inet4_pair *server_nat_pair,
                                                    const struct inet4_addr *client_addr,
                                                    __be16 client_port, u8 gress,
                                                    u16 generation_snapshot, bool use_existing_ref,
                                                    struct nat4_mapping_state_v3 *state,
                                                    struct nat_timer_value_v4_v3 **timer_value_) {
    struct nat_timer_key_v4 timer_key = {0};
    timer_key.l4proto = l4proto;
    __builtin_memcpy(&timer_key.pair_ip, server_nat_pair, sizeof(timer_key.pair_ip));

    struct nat_timer_value_v4_v3 *timer_value =
        bpf_map_lookup_elem(&nat4_mapping_timer_v3, &timer_key);
    if (timer_value) {
        *timer_value_ = timer_value;
        return TIMER_EXIST;
    }
    if (!do_new) {
        return TIMER_NOT_FOUND;
    }

    if (!use_existing_ref) {
        if (!state || nat4_v3_state_try_inc(state) != 0) {
            return TIMER_ERROR;
        }
    }

    struct nat_timer_value_v4_v3 new_value = {0};
    new_value.client_port = client_port;
    new_value.client_status = CT_INIT;
    new_value.server_status = CT_INIT;
    new_value.gress = gress;
    new_value.client_addr = *client_addr;
    new_value.create_time = bpf_ktime_get_tai_ns();
    new_value.flow_id = get_flow_id(skb->mark);
    new_value.cpu_id = bpf_get_smp_processor_id();
    new_value.ifindex = skb->ifindex;
    new_value.generation_snapshot = generation_snapshot;

    timer_value = nat4_v3_insert_ct(&timer_key, &new_value);
    if (!timer_value) {
        if (!use_existing_ref && state) {
            struct nat4_mapping_state_v3 *curr_state = nat4_v3_lookup_state(
                l4proto, server_nat_pair->dst_addr.addr, server_nat_pair->dst_port);
            if (curr_state && curr_state->generation == generation_snapshot) {
                (void)nat4_v3_state_try_dec(curr_state);
            }
        }
        return TIMER_ERROR;
    }

    *timer_value_ = timer_value;
    return TIMER_CREATED;
}

static __always_inline int nat4_v3_egress_lookup_or_new_mapping(
    struct __sk_buff *skb, u8 ip_protocol, bool allow_create_mapping,
    const struct inet4_pair *pkt_ip_pair, struct nat4_lookup_result_v3 *result) {
    u64 current_time = bpf_ktime_get_ns();
    struct nat_mapping_key_v4 egress_key = {
        .gress = NAT_MAPPING_EGRESS,
        .l4proto = ip_protocol,
        .from_port = pkt_ip_pair->src_port,
        .from_addr = pkt_ip_pair->src_addr.addr,
    };

    struct nat_mapping_value_v4 *egress_value = bpf_map_lookup_elem(&nat4_dyn_map, &egress_key);

    if (egress_value) {
        struct nat_mapping_key_v4 ingress_key = {
            .gress = NAT_MAPPING_INGRESS,
            .l4proto = ip_protocol,
            .from_addr = egress_value->addr,
            .from_port = egress_value->port,
        };
        struct nat_mapping_value_v4 *ingress_value =
            bpf_map_lookup_elem(&nat4_dyn_map, &ingress_key);
        struct nat4_mapping_state_v3 *state =
            bpf_map_lookup_elem(&nat4_dynamic_state_v3, &ingress_key);
        if (!ingress_value || !state) {
            return TC_ACT_SHOT;
        }
        egress_value->active_time = current_time;
        ingress_value->active_time = current_time;
        result->egress = egress_value;
        result->ingress = ingress_value;
        result->state = state;
        return TC_ACT_OK;
    }

    egress_value = nat4_v3_lookup_static_egress(ip_protocol, pkt_ip_pair->src_port,
                                                pkt_ip_pair->src_addr.addr);
    if (egress_value) {
        result->egress = egress_value;
        result->ingress = nat4_v3_lookup_static_ingress(ip_protocol, egress_value->port);
        result->is_static = true;
        return result->ingress ? TC_ACT_OK : TC_ACT_SHOT;
    }

    if (!allow_create_mapping) {
        return TC_ACT_SHOT;
    }

    struct wan_ip_info_key wan_search_key = {
        .ifindex = skb->ifindex,
        .l3_protocol = LANDSCAPE_IPV4_TYPE,
    };
    struct wan_ip_info_value *wan_ip_info = bpf_map_lookup_elem(&wan_ip_binding, &wan_search_key);
    if (!wan_ip_info) {
        return TC_ACT_SHOT;
    }

    struct nat4_port_queue_value_v3 alloc_item = {0};
    if (nat4_v3_alloc_port(ip_protocol, &alloc_item) != 0) {
        return TC_ACT_SHOT;
    }

    struct nat_mapping_value_v4 new_value = {
        .addr = wan_ip_info->addr.ip,
        .port = alloc_item.port,
        .trigger_addr = pkt_ip_pair->dst_addr.addr,
        .trigger_port = pkt_ip_pair->dst_port,
        .is_static = 0,
        .is_allow_reuse = get_flow_allow_reuse_port(skb->mark) ? 1 : 0,
        .active_time = current_time,
    };

    struct nat_mapping_value_v4 *egress_out = NULL;
    struct nat_mapping_value_v4 *ingress_out = NULL;
    if (nat4_v3_insert_mappings(&egress_key, &new_value, &egress_out, &ingress_out) != 0) {
        (void)nat4_v3_queue_push(ip_protocol, &alloc_item);
        return TC_ACT_SHOT;
    }

    u16 generation = alloc_item.last_generation + 1;
    if (nat4_v3_insert_state(ip_protocol, new_value.addr, new_value.port, generation) != 0) {
        nat4_v3_delete_mapping_and_state(ip_protocol, new_value.addr, new_value.port,
                                         pkt_ip_pair->src_addr.addr, pkt_ip_pair->src_port);
        (void)nat4_v3_queue_push(ip_protocol, &alloc_item);
        return TC_ACT_SHOT;
    }

    result->egress = egress_out;
    result->ingress = ingress_out;
    result->state = nat4_v3_lookup_state(ip_protocol, new_value.addr, new_value.port);
    result->alloc_item = alloc_item;
    result->created = true;
    result->is_static = false;
    return result->state ? TC_ACT_OK : TC_ACT_SHOT;
}

static __always_inline int nat4_v3_ingress_lookup_mapping(
    struct __sk_buff *skb, u8 ip_protocol, const struct inet4_pair *pkt_ip_pair,
    struct nat_mapping_value_v4 **ingress_value_, struct nat4_mapping_state_v3 **state_) {
    struct nat_mapping_key_v4 ingress_key = {
        .gress = NAT_MAPPING_INGRESS,
        .l4proto = ip_protocol,
        .from_port = pkt_ip_pair->dst_port,
        .from_addr = pkt_ip_pair->dst_addr.addr,
    };

    struct nat_mapping_value_v4 *ingress_value = bpf_map_lookup_elem(&nat4_dyn_map, &ingress_key);
    if (!ingress_value) {
        ingress_key.from_addr = 0;
        ingress_value = bpf_map_lookup_elem(&nat4_st_map, &ingress_key);
        if (!ingress_value) {
            return TC_ACT_SHOT;
        }
    }

    ingress_value->active_time = bpf_ktime_get_ns();
    *ingress_value_ = ingress_value;
    if (!ingress_value->is_static) {
        ingress_key.from_addr = pkt_ip_pair->dst_addr.addr;
        *state_ = bpf_map_lookup_elem(&nat4_dynamic_state_v3, &ingress_key);
        if (!*state_) {
            return TC_ACT_SHOT;
        }
    }
    return TC_ACT_OK;
}

#endif /* LD_NAT4_V3_H */
