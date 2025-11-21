#ifndef __LD_FLOW_H__
#define __LD_FLOW_H__
#include <bpf/bpf_helpers.h>
#include "landscape.h"
#include "packet_def.h"
#include "flow_match.h"

struct flow_mark {
    u16 trace_id;
    u8 flow_action;
    u8 flow_id;
} __attribute__((packed));

// struct flow_dns_match_key {
//     union u_inet_addr addr;
// };

struct flow_dns_match_key {
    union u_inet_addr addr;
    u8 l3_protocol;
    u8 _pad[3];
} __flow_dns_match_key;

struct flow_dns_match_value {
    u32 mark;
    u16 priority;
    u8 _pad[2];
} __flow_dns_match_value;

// 每个流中特定的 DNS 规则
struct each_flow_dns {
    __uint(type, BPF_MAP_TYPE_LRU_HASH);
    // __uint(key_size, 16);
    // __uint(map_flags, BPF_F_NO_COMMON_LRU);
    __type(key, struct flow_dns_match_key);
    __type(value, struct flow_dns_match_value);
    __uint(max_entries, 4096);
} each_flow_dns_map SEC(".maps");

// flow <-> 对应规则 map
struct {
    __uint(type, BPF_MAP_TYPE_HASH_OF_MAPS);
    __type(key, u32);
    __uint(max_entries, 512);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
    __array(values, struct each_flow_dns);
} flow_v_dns_map SEC(".maps");

//
struct flow_ip_trie_key {
    __u32 prefixlen;
    u8 l3_protocol;
    u8 _pad[3];
    u8 addr[16];
} __flow_ip_trie_key;

struct flow_ip_trie_value {
    u32 mark;
    u16 priority;
    u8 _pad[2];
} __flow_ip_trie_value;

// 每个流中特定的 目标 IP 规则
struct each_flow_ip_trie {
    __uint(type, BPF_MAP_TYPE_LPM_TRIE);
    __uint(map_flags, BPF_F_NO_PREALLOC);
    __type(key, struct flow_ip_trie_key);
    __type(value, struct flow_ip_trie_value);
    __uint(max_entries, 65536);
} each_flow_ip_map SEC(".maps");

// flow <-> 对应规则 map
struct {
    __uint(type, BPF_MAP_TYPE_HASH_OF_MAPS);
    __type(key, u32);
    __uint(max_entries, 512);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
    __array(values, struct each_flow_ip_trie);
} flow_v_ip_map SEC(".maps");

struct lan_route_key {
    __u32 prefixlen;
    u8 l3_protocol;
    u8 _pad[3];
    struct in6_addr addr;
};

struct lan_route_info {
    bool has_mac;
    u8 mac_addr[6];
    u8 _pad[2];
    u32 ifindex;
    struct in6_addr addr;
};

struct {
    __uint(type, BPF_MAP_TYPE_LPM_TRIE);
    __type(key, struct lan_route_key);
    __type(value, struct lan_route_info);
    __uint(max_entries, 1024);
    __uint(map_flags, BPF_F_NO_PREALLOC);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} rt_lan_map SEC(".maps");

struct route_target_key {
    __u32 flow_id;
    u8 l3_protocol;
    u8 _pad[3];
};

struct route_target_info {
    u32 ifindex;
    struct in6_addr gate_addr;
    // 是否有 mac
    bool has_mac;
    bool is_docker;
};

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, struct route_target_key);
    __type(value, struct route_target_info);
    __uint(max_entries, 1024);
    __uint(map_flags, BPF_F_NO_PREALLOC);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} rt_target_map SEC(".maps");

struct rt_cache_key {
    struct in6_addr local_addr;
    struct in6_addr remote_addr;
} __rt_cache_key;

struct rt_cache_value {
    union {
        __u32 mark_value;
        __u32 ifindex;
    };
} __rt_cache_value;

#define WAN_CACHE 0
#define LAN_CACHE 1

// 缓存
struct each_cache_hash {
    __uint(type, BPF_MAP_TYPE_LRU_HASH);
    __type(key, struct rt_cache_key);
    __type(value, struct rt_cache_value);
    __uint(max_entries, 65536);
} __each_cache_map SEC(".maps");

// flow <-> 对应规则 map
struct {
    __uint(type, BPF_MAP_TYPE_ARRAY_OF_MAPS);
    __type(key, u32);
    __uint(max_entries, 4);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
    __array(values, struct each_cache_hash);
} rt_cache_map SEC(".maps");

static __always_inline int setting_cache_in_wan(const struct route_context *context, u32 ifindex) {
#define BPF_LOG_TOPIC "setting_cache_in_wan"
    struct rt_cache_key search_key = {0};
    u32 key = LAN_CACHE;
    COPY_ADDR_FROM(search_key.local_addr.in6_u.u6_addr8, context->daddr.in6_u.u6_addr8);
    COPY_ADDR_FROM(search_key.remote_addr.in6_u.u6_addr8, context->saddr.in6_u.u6_addr8);

    void *lan_cache = bpf_map_lookup_elem(&rt_cache_map, &key);
    if (lan_cache) {
        struct rt_cache_value *target = bpf_map_lookup_elem(lan_cache, &search_key);
        if (target) {
            // if (context->l3_protocol == LANDSCAPE_IPV4_TYPE) {
            //     bpf_log_info("Already cached %pI4 -> %pI4", search_key.local_addr.in6_u.u6_addr8,
            //                 search_key.remote_addr.in6_u.u6_addr8);
            // } else {
            //     bpf_log_info("Already cached %pI6 -> %pI6", search_key.local_addr.in6_u.u6_addr8,
            //                 search_key.remote_addr.in6_u.u6_addr8);
            // }
            return TC_ACT_OK;
        }
    }

    key = WAN_CACHE;
    void *wan_cache = bpf_map_lookup_elem(&rt_cache_map, &key);
    if (wan_cache) {
        struct rt_cache_value *target = bpf_map_lookup_elem(wan_cache, &search_key);
        if (target) {
            target->ifindex = ifindex;
        } else {
            struct rt_cache_value new_target_cache = {0};
            new_target_cache.ifindex = ifindex;
            bpf_map_update_elem(wan_cache, &search_key, &new_target_cache, BPF_ANY);
        }

        // if (context->l3_protocol == LANDSCAPE_IPV4_TYPE) {
        //     bpf_log_info("cache %pI4 -> %pI4", search_key.local_addr.in6_u.u6_addr8,
        //                  search_key.remote_addr.in6_u.u6_addr8);
        // } else {
        //     bpf_log_info("cache %pI6 -> %pI6", search_key.local_addr.in6_u.u6_addr8,
        //                  search_key.remote_addr.in6_u.u6_addr8);
        // }
    } else {
        bpf_log_info("could not find wan_cache: %d", key);
    }

    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}

static __always_inline int setting_cache_in_lan(const struct route_context *context,
                                                u32 flow_mark) {
#define BPF_LOG_TOPIC "setting_cache_in_lan"
    struct rt_cache_key search_key = {0};
    u32 key = WAN_CACHE;
    COPY_ADDR_FROM(search_key.local_addr.in6_u.u6_addr8, context->saddr.in6_u.u6_addr8);
    COPY_ADDR_FROM(search_key.remote_addr.in6_u.u6_addr8, context->daddr.in6_u.u6_addr8);

    void *wan_cache = bpf_map_lookup_elem(&rt_cache_map, &key);
    if (wan_cache) {
        struct rt_cache_value *target = bpf_map_lookup_elem(wan_cache, &search_key);
        if (target) {
            return TC_ACT_OK;
        }
    }

    key = LAN_CACHE;
    void *lan_cache = bpf_map_lookup_elem(&rt_cache_map, &key);
    if (lan_cache) {
        struct rt_cache_value *target = bpf_map_lookup_elem(lan_cache, &search_key);
        if (target) {
            target->mark_value = flow_mark;
        } else {
            struct rt_cache_value new_target_cache = {0};
            new_target_cache.mark_value = flow_mark;
            bpf_map_update_elem(lan_cache, &search_key, &new_target_cache, BPF_ANY);
        }

        // if (context->l3_protocol == LANDSCAPE_IPV4_TYPE) {
        //     bpf_log_info("cache %pI4 -> %pI4", search_key.local_addr.in6_u.u6_addr8,
        //                  search_key.remote_addr.in6_u.u6_addr8);
        // } else {
        //     bpf_log_info("cache %pI6 -> %pI6", search_key.local_addr.in6_u.u6_addr8,
        //                  search_key.remote_addr.in6_u.u6_addr8);
        // }
    }

    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}

#endif /* __LD_FLOW_H__ */