#include "vmlinux.h"

#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#include "landscape.h"
#include "land_nat_v2.h"

char LICENSE[] SEC("license") = "Dual BSD/GPL";
const volatile u8 LOG_LEVEL = BPF_LOG_LEVEL_DEBUG;

#undef BPF_LOG_LEVEL
#undef BPF_LOG_TOPIC
#define BPF_LOG_LEVEL LOG_LEVEL

const volatile u32 current_l3_offset = 14;

const volatile u64 TCP_SYN_TIMEOUT = 1E9 * 6;
const volatile u64 TCP_TCP_TRANS = 1E9 * 60 * 4;
const volatile u64 TCP_TIMEOUT = 1E9 * 60 * 10;
const volatile u64 UDP_TIMEOUT = 1E9 * 60 * 5;

static __always_inline bool pkt_allow_initiating_ct(u8 pkt_type) {
    return pkt_type == PKT_CONNLESS_V2 || pkt_type == PKT_TCP_SYN_V2;
}

#define NAT_MAPPING_CACHE_SIZE 1024 * 64 * 2
#define NAT_MAPPING_TIMER_SIZE 1024 * 64 * 2

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, struct nat_mapping_key);
    __type(value, struct nat_mapping_value);
    __uint(max_entries, NAT_MAPPING_CACHE_SIZE);
} nat_mappings SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, struct nat_timer_key);
    __type(value, struct nat_timer_value);
    __uint(max_entries, NAT_MAPPING_TIMER_SIZE);
    __uint(map_flags, BPF_F_NO_PREALLOC);
} map_mapping_timer SEC(".maps");

volatile const u16 tcp_range_start = 32768;
// volatile const u16 tcp_range_end = 32770;
volatile const u16 tcp_range_end = 65535;

volatile const u16 udp_range_start = 32768;
volatile const u16 udp_range_end = 65535;

volatile const u16 icmp_range_start = 32768;
volatile const u16 icmp_range_end = 65535;

static __always_inline int icmpx_err_l3_offset(int l4_off) {
    return l4_off + sizeof(struct icmphdr);
}

static __always_inline void ipv4_update_csum(struct __sk_buff *skb, u32 l4_csum_off,
                                             __be32 from_addr, __be16 from_port, __be32 to_addr,
                                             __be16 to_port, bool l4_pseudo, bool l4_mangled_0) {
    bpf_l4_csum_replace(skb, l4_csum_off, from_port, to_port,
                        2 | (l4_mangled_0 ? BPF_F_MARK_MANGLED_0 : 0));
    if (l4_pseudo) {
        bpf_l4_csum_replace(skb, l4_csum_off, from_addr, to_addr,
                            4 | BPF_F_PSEUDO_HDR | (l4_mangled_0 ? BPF_F_MARK_MANGLED_0 : 0));
    }
}

static __always_inline void ipv4_update_csum_inner(struct __sk_buff *skb, u32 l4_csum_off,
                                                   __be32 from_addr, __be16 from_port,
                                                   __be32 to_addr, __be16 to_port, bool l4_pseudo,
                                                   bool l4_mangled_0) {
    u16 csum;
    if (l4_mangled_0) {
        bpf_skb_load_bytes(skb, l4_csum_off, &csum, sizeof(csum));
    }
    if (!l4_mangled_0 || csum != 0) {
        // use bpf_l3_csum_replace to avoid updating skb csum
        bpf_l3_csum_replace(skb, l4_csum_off, from_port, to_port, 2);

        if (l4_pseudo) {
            bpf_l3_csum_replace(skb, l4_csum_off, from_addr, to_addr, 4);
        }
    }
}

static __always_inline void ipv4_update_csum_icmp_err(struct __sk_buff *skb, u32 icmp_csum_off,
                                                      u32 err_ip_check_off, u32 err_l4_csum_off,
                                                      __be32 from_addr, __be16 from_port,
                                                      __be32 to_addr, __be16 to_port,
                                                      bool err_l4_pseudo, bool l4_mangled_0) {
    u16 prev_csum;
    u16 curr_csum;
    bpf_skb_load_bytes(skb, err_ip_check_off, &prev_csum, sizeof(prev_csum));

    bpf_l3_csum_replace(skb, err_ip_check_off, from_addr, to_addr, 4);

    bpf_skb_load_bytes(skb, err_ip_check_off, &curr_csum, sizeof(curr_csum));
    bpf_l4_csum_replace(skb, icmp_csum_off, prev_csum, curr_csum, 2);

    // update of inner message
#if 1
    // the update of embedded layer 4 checksum is not required but may helpful
    // for packet tracking the TCP checksum might not be included in IPv4
    // packet, check if it exists first
    if (bpf_skb_load_bytes(skb, err_l4_csum_off, &prev_csum, sizeof(prev_csum))) {
        ipv4_update_csum_inner(skb, err_l4_csum_off, from_addr, from_port, to_addr, to_port,
                               err_l4_pseudo, l4_mangled_0);

        bpf_skb_load_bytes(skb, err_l4_csum_off, &curr_csum, sizeof(curr_csum));
        bpf_l4_csum_replace(skb, icmp_csum_off, prev_csum, curr_csum, 2);
    }
#endif
    bpf_l4_csum_replace(skb, icmp_csum_off, from_addr, to_addr, 4);
    bpf_l4_csum_replace(skb, icmp_csum_off, from_port, to_port, 2);
}

static __always_inline int modify_headers(struct __sk_buff *skb, bool is_ipv4, bool is_icmpx_error,
                                          u8 nexthdr, u32 current_l3_offset, int l4_off,
                                          int err_l4_off, bool is_modify_source,
                                          union u_inet_addr *from_addr, __be16 from_port,
                                          union u_inet_addr *to_addr, __be16 to_port) {
#define BPF_LOG_TOPIC "modify_headers"

    int ret;
    int ip_offset =
        is_modify_source ? offsetof(struct iphdr, saddr) : offsetof(struct iphdr, daddr);
    ret = bpf_skb_store_bytes(skb, current_l3_offset + ip_offset, &to_addr->ip, sizeof(to_addr->ip),
                              0);

    if (ret) {
        return ret;
    }
    ret = bpf_l3_csum_replace(skb, current_l3_offset + offsetof(struct iphdr, check), from_addr->ip,
                              to_addr->ip, 4);
    if (ret) {
        return ret;
    }
    if (l4_off == 0) {
        return 0;
    }

    int l4_to_port_off;
    int l4_to_check_off;
    bool l4_check_pseudo;
    bool l4_check_mangle_0;
    switch (nexthdr) {
    case IPPROTO_TCP:
        l4_to_port_off = is_modify_source ^ is_icmpx_error ? offsetof(struct tcphdr, source)
                                                           : offsetof(struct tcphdr, dest);
        l4_to_check_off = offsetof(struct tcphdr, check);
        l4_check_pseudo = true;
        l4_check_mangle_0 = false;
        break;
    case IPPROTO_UDP:
        l4_to_port_off = is_modify_source ^ is_icmpx_error ? offsetof(struct udphdr, source)
                                                           : offsetof(struct udphdr, dest);
        l4_to_check_off = offsetof(struct udphdr, check);
        l4_check_pseudo = true;
        l4_check_mangle_0 = is_ipv4;
        break;
    case IPPROTO_ICMP:
        l4_to_port_off = offsetof(struct icmphdr, un.echo.id);
        l4_to_check_off = offsetof(struct icmphdr, checksum);
        l4_check_pseudo = !is_ipv4;
        l4_check_mangle_0 = false;
        break;
    default:
        return 1;
    }

    if (is_icmpx_error) {
        int icmpx_error_offset =
            is_modify_source ? offsetof(struct iphdr, daddr) : offsetof(struct iphdr, saddr);
        ret = bpf_write_inet_addr(skb, is_ipv4, icmpx_err_l3_offset(l4_off) + icmpx_error_offset,
                                  to_addr);
        if (ret) {
            return ret;
        }
    }

    ret = bpf_write_port(skb, (is_icmpx_error ? err_l4_off : l4_off) + l4_to_port_off, to_port);
    if (ret) {
        return ret;
    }

    if (is_icmpx_error) {
        if (is_ipv4) {
            ipv4_update_csum_icmp_err(skb, l4_off + offsetof(struct icmphdr, checksum),
                                      icmpx_err_l3_offset(l4_off) + offsetof(struct iphdr, check),
                                      err_l4_off + l4_to_check_off, from_addr->ip, from_port,
                                      to_addr->ip, to_port, l4_check_pseudo, l4_check_mangle_0);
        }
    } else {
        if (is_ipv4) {
            // __be16 check_sum;
            // bpf_skb_load_bytes(skb, l4_off + l4_to_check_off, &check_sum, sizeof(u16));
            // bpf_log_info("tcphdr before update checksum is: %u", bpf_ntohs(check_sum));

            ipv4_update_csum(skb, l4_off + l4_to_check_off, from_addr->ip, from_port, to_addr->ip,
                             to_port, l4_check_pseudo, l4_check_mangle_0);

            // bpf_skb_load_bytes(skb, l4_off + l4_to_check_off, &check_sum, sizeof(u16));
            // bpf_log_info("tcphdr before update checksum is: %u", bpf_ntohs(check_sum));
        }
    }

    return 0;
#undef BPF_LOG_TOPIC
}

static __always_inline bool ct_change_state(struct nat_timer_value *timer_track_value,
                                            u64 curr_state, u64 next_state) {
    return __sync_bool_compare_and_swap(&timer_track_value->status, curr_state, next_state);
}

static __always_inline int ct_reset_timer(struct nat_timer_value *timer_track_value, u64 timeout) {
#define BPF_LOG_TOPIC "ct_reset_timer"
    // bpf_log_info("ct_reset_timer : %llu", timeout);
    return bpf_timer_start(&timer_track_value->timer, timeout, 0);
#undef BPF_LOG_TOPIC
}

static __always_inline int ct_state_transition(u8 l4proto, u8 pkt_type, u8 gress,
                                               struct nat_timer_value *ct_timer_value) {
#define BPF_LOG_TOPIC "ct_state_transition"
    u64 curr_state = ct_timer_value->status;

#define NEW_STATE(__state)                                                                         \
    if (!ct_change_state(ct_timer_value, curr_state, (__state))) {                                 \
        return TC_ACT_SHOT;                                                                        \
    }
#define RESET_TIMER(__timeout) ct_reset_timer(ct_timer_value, (__timeout))

    if (pkt_type == PKT_CONNLESS_V2) {
        NEW_STATE(OTHER_EST);
        RESET_TIMER(UDP_TIMEOUT);
        return TC_ACT_OK;
    }

    if (pkt_type == PKT_TCP_RST_V2) {
        NEW_STATE(TIMER_INIT);
        RESET_TIMER(TCP_SYN_TIMEOUT);
        return TC_ACT_OK;
    }

    if (pkt_type == PKT_TCP_SYN_V2) {
        NEW_STATE(TIMER_INIT);
        if (gress == ct_timer_value->gress) {
            RESET_TIMER(TCP_SYN_TIMEOUT);
        } else {
            RESET_TIMER(TCP_TCP_TRANS);
        }
        return TC_ACT_OK;
    }

    RESET_TIMER(TCP_TIMEOUT);

    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}

static int timer_clean_callback(void *map_mapping_timer_, struct nat_timer_key *key,
                                struct nat_timer_value *value) {
#define BPF_LOG_TOPIC "timer_clean_callback"

    // bpf_log_info("timer_clean_callback: %d", bpf_ntohs(value->trigger_port));
    struct nat_mapping_key egress_mapping_key = {
        .l4proto = key->l4proto,
        .gress = NAT_MAPPING_EGRESS,
        .from_addr = key->pair_ip.src_addr,
        .from_port = key->pair_ip.src_port,
    };

    struct nat_mapping_key ingress_mapping_key = {
        .l4proto = key->l4proto,
        .gress = NAT_MAPPING_INGRESS,
        .from_addr = key->pair_ip.dst_addr,
        .from_port = key->pair_ip.dst_port,
    };

    bpf_map_delete_elem(&nat_mappings, &egress_mapping_key);
    bpf_map_delete_elem(&nat_mappings, &ingress_mapping_key);

    bpf_map_delete_elem(&map_mapping_timer, key);
    return 0;
#undef BPF_LOG_TOPIC
}

static __always_inline struct nat_timer_value *
insert_new_nat_timer(u8 l4proto, const struct nat_timer_key *key,
                     const struct nat_timer_value *val) {
#define BPF_LOG_TOPIC "insert_new_nat_timer"
    // bpf_log_info("protocol: %u, src_port: %u -> dst_port: %u", l4proto,
    // bpf_ntohs(key->pair_ip.src_port), bpf_ntohs(key->pair_ip.dst_port)); bpf_log_info("src_ip:
    // %lu -> dst_ip: %lu", bpf_ntohl(key->pair_ip.src_addr.ip),
    // bpf_ntohl(key->pair_ip.dst_addr.ip));

    int ret = bpf_map_update_elem(&map_mapping_timer, key, val, BPF_NOEXIST);
    if (ret) {
        bpf_log_error("failed to insert conntrack entry, err:%d", ret);
        return NULL;
    }
    struct nat_timer_value *value = bpf_map_lookup_elem(&map_mapping_timer, key);
    if (!value) return NULL;

    ret = bpf_timer_init(&value->timer, &map_mapping_timer, CLOCK_MONOTONIC);
    if (ret) {
        goto delete_timer;
    }
    ret = bpf_timer_set_callback(&value->timer, timer_clean_callback);
    if (ret) {
        goto delete_timer;
    }
    ret = bpf_timer_start(&value->timer, l4proto == IPPROTO_TCP ? TCP_TIMEOUT : UDP_TIMEOUT, 0);
    if (ret) {
        goto delete_timer;
    }

    return value;
delete_timer:
    bpf_log_error("setup timer err:%d", ret);
    bpf_map_delete_elem(&map_mapping_timer, key);
    return NULL;
#undef BPF_LOG_TOPIC
}

static __always_inline int lookup_or_new_ct(u8 l4proto, bool do_new,
                                            const struct inet_pair *pkt_ip_pair,
                                            struct nat_mapping_value *nat_egress_value,
                                            struct nat_mapping_value *nat_ingress_value,
                                            struct nat_timer_value **timer_value_) {
#define BPF_LOG_TOPIC "lookup_or_new_ct"

    struct nat_timer_key timer_key = {
        .l4proto = l4proto,
        ._pad = {0, 0, 0},
        .pair_ip =
            {
                .src_port = nat_ingress_value->port,
                .dst_port = nat_egress_value->port,
            },
    };
    COPY_ADDR_FROM(timer_key.pair_ip.src_addr.all, nat_ingress_value->addr.all);
    COPY_ADDR_FROM(timer_key.pair_ip.dst_addr.all, nat_egress_value->addr.all);

    // bpf_log_info("protocol: %u, src_port: %u -> dst_port: %u", l4proto,
    //              bpf_ntohs(timer_key.pair_ip.src_port), bpf_ntohs(timer_key.pair_ip.dst_port));
    // bpf_log_info("src_ip: %lu -> dst_ip: %lu", bpf_ntohl(timer_key.pair_ip.src_addr.ip),
    //              bpf_ntohl(timer_key.pair_ip.dst_addr.ip));

    struct nat_timer_value *timer_value = bpf_map_lookup_elem(&map_mapping_timer, &timer_key);
    // bpf_log_info("timer_value: %u", timer_value);
    if (timer_value) {
        *timer_value_ = timer_value;
        return TIMER_EXIST;
    }
    if (!timer_value && !do_new) {
        return TIMER_NOT_FOUND;
    }

    struct nat_timer_value timer_value_new = {0};
    timer_value_new.trigger_port = nat_ingress_value->trigger_port;
    timer_value_new.status = TIMER_INIT;
    timer_value_new.gress = NAT_MAPPING_EGRESS;
    COPY_ADDR_FROM(timer_value_new.trigger_saddr.all, nat_egress_value->trigger_addr.all);
    timer_value = insert_new_nat_timer(l4proto, &timer_key, &timer_value_new);
    if (timer_value == NULL) {
        return TIMER_ERROR;
    }
    // bpf_log_debug("insert new CT");

    // 发送 event
    struct nat_conn_event *event;
    event = bpf_ringbuf_reserve(&nat_conn_events, sizeof(struct nat_conn_event), 0);
    if (event != NULL) {
        COPY_ADDR_FROM(event->dst_addr.all, nat_egress_value->trigger_addr.all);
        COPY_ADDR_FROM(event->src_addr.all, nat_ingress_value->addr.all);
        event->src_port = nat_ingress_value->port;
        event->dst_port = nat_egress_value->trigger_port;
        event->l4_proto = l4proto;
        event->l3_proto = LANDSCAPE_IPV4_TYPE;
        event->flow_id = 0;
        event->trace_id = 0;
        event->time = bpf_ktime_get_ns();
        event->event_type = NAT_CREATE_CONN;
        bpf_ringbuf_submit(event, 0);
    }

    *timer_value_ = timer_value;
    return TIMER_CREATED;
#undef BPF_LOG_TOPIC
}

static __always_inline struct nat_mapping_value *
insert_mappings(const struct nat_mapping_key *key, const struct nat_mapping_value *val,
                struct nat_mapping_value **lk_val_rev) {
#define BPF_LOG_TOPIC "insert_mappings"
    int ret;
    struct nat_mapping_key key_rev = {
        .gress = key->gress ^ GRESS_MASK,
        .l4proto = key->l4proto,
        .from_addr = val->addr,
        .from_port = val->port,
    };

    struct nat_mapping_value val_rev = {
        .port = key->from_port,
        .addr = key->from_addr,
        .trigger_addr = val->trigger_addr,
        .trigger_port = val->trigger_port,
        .active_time = val->active_time,
        ._pad = {0, 0, 0},
    };

    ret = bpf_map_update_elem(&nat_mappings, key, val, BPF_ANY);
    if (ret) {
        bpf_log_error("failed to insert binding entry, err:%d", ret);
        goto error_update;
    }
    ret = bpf_map_update_elem(&nat_mappings, &key_rev, &val_rev, BPF_ANY);
    if (ret) {
        bpf_log_error("failed to insert reverse binding entry, err:%d", ret);
        goto error_update;
    }

    if (lk_val_rev) {
        *lk_val_rev = bpf_map_lookup_elem(&nat_mappings, &key_rev);
        if (!*lk_val_rev) {
            return NULL;
        }
    }

    return bpf_map_lookup_elem(&nat_mappings, key);
error_update:
    bpf_map_delete_elem(&nat_mappings, key);
    bpf_map_delete_elem(&nat_mappings, &key_rev);
    return NULL;
#undef BPF_LOG_TOPIC
}

static int search_port_callback(u32 index, struct search_port_ctx *ctx) {
#define BPF_LOG_TOPIC "search_port_callback"
    ctx->ingress_key.from_port = bpf_htons(ctx->curr_port);
    struct nat_mapping_value *value = bpf_map_lookup_elem(&nat_mappings, &ctx->ingress_key);
    u64 current_time = bpf_ktime_get_ns();
    // 大于协议的超时时间
    if (!value || (current_time - value->active_time) > ctx->timeout_interval) {
        ctx->found = true;
        return BPF_LOOP_RET_BREAK;
    }

    if (ctx->curr_port != ctx->range.end) {
        ctx->curr_port++;
    } else {
        ctx->curr_port = ctx->range.start;
    }
    if (--ctx->remaining_size == 0) {
        return BPF_LOOP_RET_BREAK;
    }

    return BPF_LOOP_RET_CONTINUE;
#undef BPF_LOG_TOPIC
}

static __always_inline int
ingress_lookup_or_new_mapping(struct __sk_buff *skb, u8 ip_protocol, bool allow_create_mapping,
                              const struct inet_pair *pkt_ip_pair,
                              struct nat_mapping_value **nat_egress_value_,
                              struct nat_mapping_value **nat_ingress_value_) {
#define BPF_LOG_TOPIC "ingress_lookup_or_new_mapping"
    if (pkt_ip_pair == NULL) {
        return TC_ACT_SHOT;
    }
    //
    struct nat_mapping_key ingress_key = {
        .gress = NAT_MAPPING_INGRESS,
        .l4proto = ip_protocol,              // 原有的 l4 层协议值
        .from_port = pkt_ip_pair->dst_port,  // 数据包中的 内网端口
        .from_addr = pkt_ip_pair->dst_addr,
    };

    // 倒置的值
    struct nat_mapping_value *nat_ingress_value = bpf_map_lookup_elem(&nat_mappings, &ingress_key);
    struct nat_mapping_value *nat_egress_value = NULL;
    if (!nat_ingress_value) {
        if (!allow_create_mapping) {
            return TC_ACT_SHOT;
        }
        return TC_ACT_SHOT;
    } else {
        // 已经存在就查询另外一个值 并进行刷新时间
        struct nat_mapping_key egress_key = {
            .gress = NAT_MAPPING_EGRESS,
            .l4proto = ip_protocol,                // 原有的 l4 层协议值
            .from_port = nat_ingress_value->port,  // 数据包中的 内网端口
            .from_addr = nat_ingress_value->addr,  // 内网原始地址
        };
        nat_egress_value = bpf_map_lookup_elem(&nat_mappings, &egress_key);

        if (!nat_egress_value) {
            return TC_ACT_SHOT;
        }
        nat_ingress_value->active_time = bpf_ktime_get_ns();
        nat_egress_value->active_time = bpf_ktime_get_ns();
    }

    *nat_egress_value_ = nat_egress_value;
    *nat_ingress_value_ = nat_ingress_value;
    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}

static __always_inline int
egress_lookup_or_new_mapping(struct __sk_buff *skb, u8 ip_protocol, bool allow_create_mapping,
                             const struct inet_pair *pkt_ip_pair,
                             struct nat_mapping_value **nat_egress_value_,
                             struct nat_mapping_value **nat_ingress_value_) {
#define BPF_LOG_TOPIC "egress_lookup_or_new_mapping"
    //
    struct nat_mapping_key egress_key = {
        .gress = NAT_MAPPING_EGRESS,
        .l4proto = ip_protocol,              // 原有的 l4 层协议值
        .from_port = pkt_ip_pair->src_port,  // 数据包中的 内网端口
        .from_addr = pkt_ip_pair->src_addr,  // 内网原始地址
    };

    // 倒置的值
    struct nat_mapping_value *nat_ingress_value = NULL;
    struct nat_mapping_value *nat_egress_value = bpf_map_lookup_elem(&nat_mappings, &egress_key);
    if (!nat_egress_value) {
        if (!allow_create_mapping) {
            return TC_ACT_SHOT;
        }
        struct wan_ip_info_key wan_search_key = {0};
        wan_search_key.ifindex = skb->ifindex;
        wan_search_key.l3_protocol = LANDSCAPE_IPV4_TYPE;

        struct wan_ip_info_value *wan_ip_info =
            bpf_map_lookup_elem(&wan_ipv4_binding, &wan_search_key);

        if (!wan_ip_info) {
            bpf_log_info("can't find the wan ip, using ifindex: %d", skb->ifindex);
            return TC_ACT_SHOT;
        }
        bool allow_reuse_port = get_flow_allow_reuse_port(skb->mark);
        struct nat_mapping_value new_nat_egress_value = {0};

        new_nat_egress_value.addr.ip = wan_ip_info->addr.ip;
        new_nat_egress_value.port = egress_key.from_port;  // 尽量先试试使用客户端发起时候的端口
        new_nat_egress_value.trigger_addr = pkt_ip_pair->dst_addr;
        new_nat_egress_value.trigger_port = pkt_ip_pair->dst_port;
        new_nat_egress_value.is_static = 0;
        new_nat_egress_value.active_time = bpf_ktime_get_ns();
        new_nat_egress_value.is_allow_reuse = allow_reuse_port ? 1 : 0;

        int ret;
        struct search_port_ctx ctx = {
            .ingress_key =
                {
                    .gress = NAT_MAPPING_INGRESS,
                    .l4proto = ip_protocol,
                    .from_addr = new_nat_egress_value.addr,
                    .from_port = new_nat_egress_value.port,
                },
            .curr_port = bpf_ntohs(new_nat_egress_value.port),
            .found = false,
        };

        if (ip_protocol == IPPROTO_TCP) {
            ctx.range.start = tcp_range_start;
            ctx.range.end = tcp_range_end;
            ctx.remaining_size = tcp_range_end - tcp_range_start;
            ctx.timeout_interval = TCP_TCP_TRANS;
        } else if (ip_protocol == IPPROTO_UDP) {
            ctx.range.start = udp_range_start;
            ctx.range.end = udp_range_end;
            ctx.remaining_size = udp_range_end - udp_range_start;
            ctx.timeout_interval = UDP_TIMEOUT;
        } else if (ip_protocol == IPPROTO_ICMP) {
            ctx.range.start = icmp_range_start;
            ctx.range.end = icmp_range_end;
            ctx.remaining_size = icmp_range_end - icmp_range_start;
            ctx.timeout_interval = UDP_TIMEOUT;
        }

        if (ctx.remaining_size == 0) {
            bpf_log_error("not free port range start: %d end: %d", ctx.range.start, ctx.range.end);
            return TC_ACT_SHOT;
        }

        if (ctx.curr_port < ctx.range.start || ctx.curr_port > ctx.range.end) {
            u16 index = ctx.curr_port % ctx.remaining_size;
            ctx.curr_port = ctx.range.start + index;
        }

        ret = bpf_loop(65536, search_port_callback, &ctx, 0);
        if (ret < 0) {
            return TC_ACT_SHOT;
        }

        if (ctx.found) {
            new_nat_egress_value.port = ctx.ingress_key.from_port;
            // bpf_log_debug("found free binding %d -> %d", bpf_ntohs(egress_key.from_port),
            //               bpf_ntohs(new_nat_egress_value.port));
        } else {
            bpf_log_debug("mapping is full");
            return TC_ACT_SHOT;
        }
        nat_egress_value = insert_mappings(&egress_key, &new_nat_egress_value, &nat_ingress_value);
        if (!nat_egress_value) {
            return TC_ACT_SHOT;
        }
    } else {
        // 已经存在就查询另外一个值 并进行刷新时间
        struct nat_mapping_key ingress_key = {
            .gress = NAT_MAPPING_INGRESS,
            .l4proto = ip_protocol,               // 原有的 l4 层协议值
            .from_port = nat_egress_value->port,  // 数据包中的 内网端口
            .from_addr = nat_egress_value->addr,  // 内网原始地址
        };
        nat_ingress_value = bpf_map_lookup_elem(&nat_mappings, &ingress_key);

        if (!nat_ingress_value) {
            return TC_ACT_SHOT;
        }
        nat_ingress_value->active_time = bpf_ktime_get_ns();
        nat_egress_value->active_time = bpf_ktime_get_ns();
    }

    *nat_egress_value_ = nat_egress_value;
    *nat_ingress_value_ = nat_ingress_value;
    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}

static __always_inline int lookup_static_mapping(struct __sk_buff *skb, u8 ip_protocol, u8 gress,
                                                 const struct inet_pair *pkt_ip_pair,
                                                 struct nat_mapping_value **nat_ingress_value_,
                                                 struct nat_mapping_value **nat_egress_value_) {
#define BPF_LOG_TOPIC "lookup_static_mapping"
    struct static_nat_mapping_key egress_key = {0};
    struct static_nat_mapping_key ingress_key = {0};

    egress_key.l3_protocol = LANDSCAPE_IPV4_TYPE;
    egress_key.l4_protocol = ip_protocol;
    ingress_key.l3_protocol = LANDSCAPE_IPV4_TYPE;
    ingress_key.l4_protocol = ip_protocol;

    struct nat_mapping_value *nat_gress_value = NULL;
    struct nat_mapping_value *nat_gress_value_rev = NULL;
    if (gress == NAT_MAPPING_EGRESS) {
        egress_key.gress = NAT_MAPPING_EGRESS;
        egress_key.prefixlen = 192;
        egress_key.port = pkt_ip_pair->src_port;
        COPY_ADDR_FROM(egress_key.addr.all, pkt_ip_pair->src_addr.all);

        // 倒置的值
        nat_gress_value = bpf_map_lookup_elem(&static_nat_mappings, &egress_key);
        if (nat_gress_value) {
            // bpf_log_info("find egress value: nat_port: %u", bpf_htons(nat_gress_value->port));
            *nat_egress_value_ = nat_gress_value;
        } else {
            // bpf_log_info("can't find egress value: %u", bpf_htons(egress_key.port));
            return TC_ACT_SHOT;
        }
    } else {
        ingress_key.prefixlen = 96;
        ingress_key.gress = NAT_MAPPING_INGRESS;
        ingress_key.port = pkt_ip_pair->dst_port;
        // using current ifindex to query
        // egress_key.addr.ip = skb->ifindex;
        nat_gress_value_rev = bpf_map_lookup_elem(&static_nat_mappings, &ingress_key);

        if (!nat_gress_value_rev) {
            // bpf_log_info("can't find ingress key: target port: %u, protocol: %u",
            // bpf_htons(ingress_key.port), ip_protocol);
            return TC_ACT_SHOT;
        }
        // bpf_log_info("find ingress value: target %pI4:%u", nat_gress_value_rev->addr.all,
        //              bpf_htons(nat_gress_value_rev->port));
        *nat_ingress_value_ = nat_gress_value_rev;
    }

    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}


SEC("tc/ingress")
int ingress_nat(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC ">>> ingress_nat >>>"
    // struct ip_packet_info_v2 packet_info = {0};
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

    if (pkg_offset.l3_protocol == LANDSCAPE_IPV6_TYPE) {
        return ipv6_ingress_prefix_check_and_replace(skb, &pkg_offset, &ip_pair);
    }

    bool is_icmpx_error = is_icmp_error_pkt(&pkg_offset);
    bool allow_create_mapping = pkg_offset.l4_protocol == IPPROTO_ICMP;

    // egress  存储的是 Ac:Pc -> An:Pn 的值
    // ingress 存储的是 An:Pn -> Ac:Pc 的值
    struct nat_mapping_value *nat_egress_value, *nat_ingress_value;

    // 先检查是否有静态映射
    ret = lookup_static_mapping(skb, pkg_offset.l4_protocol, NAT_MAPPING_INGRESS, &ip_pair,
                                &nat_ingress_value, &nat_egress_value);
    if (ret != TC_ACT_OK) {
        ret = ingress_lookup_or_new_mapping(skb, pkg_offset.l4_protocol, allow_create_mapping,
                                            &ip_pair, &nat_egress_value, &nat_ingress_value);

        if (ret != TC_ACT_OK) {
            return TC_ACT_SHOT;
        }

        // bpf_log_info("ingress value, %pI4 : %u", &nat_ingress_value->addr,
        //              bpf_ntohs(nat_ingress_value->port));
        // bpf_log_info("egress  value, %pI4 : %u", &nat_egress_value->addr.ip,
        //              bpf_ntohs(nat_egress_value->port));

        if (!nat_egress_value->is_static) {
            struct nat_timer_value *ct_timer_value;
            ret = lookup_or_new_ct(pkg_offset.l4_protocol, allow_create_mapping, &ip_pair,
                                   nat_egress_value, nat_ingress_value, &ct_timer_value);
            if (ret == TIMER_NOT_FOUND || ret == TIMER_ERROR) {
                bpf_log_info("connect ret :%u", ret);
                return TC_ACT_SHOT;
            }
            if (!is_icmpx_error || ct_timer_value != NULL) {
                ct_state_transition(pkg_offset.l4_protocol, pkg_offset.pkt_type, NAT_MAPPING_EGRESS,
                                    ct_timer_value);
            }
        }
        // } else {
        //     bpf_log_info("packet dst port: %u -> %u", bpf_ntohs(ip_pair.src_port),
        //                  bpf_ntohs(ip_pair.dst_port));
        //     bpf_log_info("modify dst port:  %u -> %u", bpf_ntohs(ip_pair.src_port),
        //                  bpf_ntohs(nat_ingress_value->port));

        //     bpf_log_info("src IP: %pI4,", &ip_pair.src_addr);
        //     bpf_log_info("dst IP: %pI4,", &ip_pair.dst_addr);
        //     bpf_log_info("real IP: %pI4,", &nat_ingress_value->addr);
    }

    if (nat_ingress_value == NULL) {
        bpf_log_info("nat_ingress_value is null");
        return TC_ACT_SHOT;
    }

    union u_inet_addr lan_ip;
    if (nat_ingress_value->is_static && nat_ingress_value->addr.ip == 0) {
        COPY_ADDR_FROM(lan_ip.all, ip_pair.dst_addr.all);
    } else {
        COPY_ADDR_FROM(lan_ip.all, nat_ingress_value->addr.all);
    }

    // if (nat_ingress_value->is_static && nat_ingress_value->addr.ip != 0) {
    //     bpf_log_info("lan_ip IP: %pI4:%u", &lan_ip.all, bpf_ntohs(nat_ingress_value->port));
    // }

    // bpf_log_info("nat_ip IP: %pI4:%u", &lan_ip.all, bpf_ntohs(nat_ingress_value->port));

    // modify source
    ret = modify_headers(skb, true, is_icmpx_error, pkg_offset.l4_protocol, current_l3_offset,
                         pkg_offset.l4_offset, pkg_offset.icmp_error_inner_l4_offset, false,
                         &ip_pair.dst_addr, ip_pair.dst_port, &lan_ip, nat_ingress_value->port);
    if (ret) {
        bpf_log_error("failed to update csum, err:%d", ret);
        return TC_ACT_SHOT;
    }

    return TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}

SEC("tc/egress")
int egress_nat(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "<<< egress_nat <<<"

    // struct ip_packet_info_v2 packet_info = {0};

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

    if (pkg_offset.l3_protocol == LANDSCAPE_IPV6_TYPE) {
        return ipv6_egress_prefix_check_and_replace(skb, &pkg_offset, &ip_pair);
    }

    // bpf_log_info("packet :%pI4 : %u -> %pI4 : %u", ip_pair.src_addr.all,
    //              bpf_ntohs(ip_pair.src_port), ip_pair.dst_addr.all, bpf_ntohs(ip_pair.dst_port));

    // bpf_log_info("packet pkt_type: %d", packet_info.pkt_type);
    // bpf_log_info("icmp_error_payload_offset: %d", packet_info.icmp_error_payload_offset);

    bool is_icmpx_error = is_icmp_error_pkt(&pkg_offset);
    bool allow_create_mapping = !is_icmpx_error && pkt_allow_initiating_ct(pkg_offset.pkt_type);

    // bpf_log_info("is is_icmpx_error", ip_pair.src_addr.all,
    //              bpf_ntohs(ip_pair.src_port), ip_pair.dst_addr.all, bpf_ntohs(ip_pair.dst_port));

    // egress  存储的是 Ac:Pc -> An:Pn 的值
    // ingress 存储的是 An:Pn -> Ac:Pc 的值
    struct nat_mapping_value *nat_egress_value, *nat_ingress_value;

    // bpf_log_info("allow_create_mapping : %d", allow_create_mapping);

    ret = lookup_static_mapping(skb, pkg_offset.l4_protocol, NAT_MAPPING_EGRESS, &ip_pair,
                                &nat_ingress_value, &nat_egress_value);

    if (ret != TC_ACT_OK) {
        ret = egress_lookup_or_new_mapping(skb, pkg_offset.l4_protocol, allow_create_mapping,
                                           &ip_pair, &nat_egress_value, &nat_ingress_value);

        if (ret != TC_ACT_OK) {
            return TC_ACT_SHOT;
        }

        // bool allow_reuse_port = get_flow_allow_reuse_port(skb->mark);
        // if (allow_reuse_port) {
        //     bpf_log_info("allow_reuse_port: %u, skb->mark: %u", allow_reuse_port, skb->mark);
        // }
        if (nat_egress_value->is_allow_reuse == 0 && pkg_offset.l4_protocol != IPPROTO_ICMP) {
            // PORT REUSE check
            if (!ip_addr_equal(&ip_pair.dst_addr, &nat_egress_value->trigger_addr) ||
                ip_pair.dst_port != nat_egress_value->trigger_port) {
                bpf_log_info("FLOW_ALLOW_REUSE MARK not set, DROP PACKET");
                bpf_log_info("dst IP: %pI4,", &ip_pair.dst_addr);
                bpf_log_info("trigger_addr IP: %pI4,", &nat_egress_value->trigger_addr);
                bpf_log_info("compare ip result: %d",
                             ip_addr_equal(&ip_pair.dst_addr, &nat_egress_value->trigger_addr));
                bpf_log_info("trigger_port: %u,", bpf_ntohs(nat_egress_value->trigger_port));
                bpf_log_info("dst_port: %u,", bpf_ntohs(ip_pair.dst_port));
                bpf_log_info("compare port result: %d",
                             ip_pair.dst_port == nat_egress_value->trigger_port);
                return TC_ACT_SHOT;
            }
        }

        // bpf_log_info("ingress value, %pI4 : %u", &nat_ingress_value->addr,
        //              bpf_ntohs(nat_ingress_value->port));
        // bpf_log_info("egress  value, %pI4 : %u", &nat_egress_value->addr.ip,
        //              bpf_ntohs(nat_egress_value->port));

        if (!nat_egress_value->is_static) {
            struct nat_timer_value *ct_timer_value;
            ret = lookup_or_new_ct(pkg_offset.l4_protocol, allow_create_mapping, &ip_pair,
                                   nat_egress_value, nat_ingress_value, &ct_timer_value);
            if (ret == TIMER_NOT_FOUND || ret == TIMER_ERROR) {
                return TC_ACT_SHOT;
            }
            if (!is_icmpx_error || ct_timer_value != NULL) {
                ct_state_transition(pkg_offset.l4_protocol, pkg_offset.pkt_type, NAT_MAPPING_EGRESS,
                                    ct_timer_value);
            }
        }
    }

    // bpf_log_info("packet src port: %u -> %u", bpf_ntohs(ip_pair.src_port),
    //              bpf_ntohs(ip_pair.dst_port));
    // bpf_log_info("modify src port:  %u -> %u", bpf_ntohs(nat_egress_value->port),
    //              bpf_ntohs(ip_pair.dst_port));

    // bpf_log_info("src IP: %pI4,", &ip_pair.src_addr);
    // bpf_log_info("dst IP: %pI4,", &ip_pair.dst_addr);
    // bpf_log_info("mapping IP: %pI4,", &nat_egress_value->addr);

    if (nat_egress_value == NULL) {
        bpf_log_info("nat_egress_value is null");
        return TC_ACT_SHOT;
    }

    union u_inet_addr nat_addr;
    if (nat_egress_value->is_static) {
        struct wan_ip_info_key wan_search_key = {0};
        wan_search_key.ifindex = skb->ifindex;
        wan_search_key.l3_protocol = LANDSCAPE_IPV4_TYPE;

        struct wan_ip_info_value *wan_ip_info =
            bpf_map_lookup_elem(&wan_ipv4_binding, &wan_search_key);
        if (!wan_ip_info) {
            bpf_log_info("can't find the wan ip, using ifindex: %d", skb->ifindex);
            return TC_ACT_SHOT;
        }
        nat_addr.ip = wan_ip_info->addr.ip;
    } else {
        COPY_ADDR_FROM(nat_addr.all, nat_egress_value->addr.all);
    }

    // bpf_log_info("nat_ip IP: %pI4:%u", &nat_addr.all, bpf_ntohs(nat_egress_value->port));

    // modify source
    ret = modify_headers(skb, true, is_icmpx_error, pkg_offset.l4_protocol, current_l3_offset,
                         pkg_offset.l4_offset, pkg_offset.icmp_error_inner_l4_offset, true,
                         &ip_pair.src_addr, ip_pair.src_port, &nat_addr, nat_egress_value->port);
    if (ret) {
        bpf_log_error("failed to update csum, err:%d", ret);
        return TC_ACT_SHOT;
    }

    return TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}

SEC("tc/egress")
int test_nat_read(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "<<< test_nat_read <<<"

    // struct packet_offset_info pkg_offset = {0};
    // struct inet_pair ip_pair;
    struct ip_packet_info_v2 packet_info = {0};
    int ret = 0;

    ret = scan_packet(skb, current_l3_offset, &packet_info.offset);
    if (ret) {
        return ret;
    }

    ret = read_packet_info(skb, &packet_info.offset, &packet_info.pair_ip);
    if (ret) {
        return ret;
    }

    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}

SEC("tc/egress")
int handle_ipv6_egress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "<<< handle_ipv6_egress <<<"

    struct ip_packet_info_v2 packet_info = {0};
    int ret = 0;

    ret = scan_packet(skb, current_l3_offset, &packet_info.offset);
    if (ret) {
        return ret;
    }

    ret = read_packet_info(skb, &packet_info.offset, &packet_info.pair_ip);
    if (ret) {
        return ret;
    }

    ret = ipv6_egress_prefix_check_and_replace(skb, &packet_info.offset, &packet_info.pair_ip);
    if (ret) {
        return ret;
    }

    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}

SEC("tc/ingress")
int handle_ipv6_ingress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "<<< handle_ipv6_ingress <<<"

    struct ip_packet_info_v2 packet_info = {0};
    int ret = 0;

    ret = scan_packet(skb, current_l3_offset, &packet_info.offset);
    if (ret) {
        return ret;
    }

    ret = read_packet_info(skb, &packet_info.offset, &packet_info.pair_ip);
    if (ret) {
        return ret;
    }

    ret = ipv6_ingress_prefix_check_and_replace(skb, &packet_info.offset, &packet_info.pair_ip);
    if (ret) {
        return ret;
    }

    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}