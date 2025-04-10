#ifndef __LD_FLOW_H__
#define __LD_FLOW_H__
#include <bpf/bpf_helpers.h>
#include "landscape.h"
#include "packet_def.h"

enum flow_chache_type {
    PASS = 0,
    REDIRECT = 1,
};

struct flow_match_key {
    // 源 mac 地址
    unsigned char h_source[6];
    // vlan id
    u32 vlan_tci;
    // tos value
    u8 tos;
    u8 _pad;
};

struct flow_cache_key {
    union u_inet_addr src_addr;
    union u_inet_addr dst_addr;
    struct flow_match_key match_key;
};

struct flow_target_info {
    // 目标 index
    u32 ifindex;
    // 是否有 mac
    bool has_mac;
    bool is_docker;
};

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(map_flags, BPF_F_NO_PREALLOC);
    __type(key, u32);                        // flow_id
    __type(value, struct flow_target_info);  // target ifindex
    __uint(max_entries, 2048);
} flow_target_map SEC(".maps");

// struct each_flow_target {
//     __uint(type, BPF_MAP_TYPE_HASH);
//     __uint(map_flags, BPF_F_NO_PREALLOC);
//     __type(key, u32);
//     __type(value, struct flow_target_info);
//     __uint(max_entries, 2048);
// } each_flow_target_map SEC(".maps");

// struct {
//     __uint(type, BPF_MAP_TYPE_HASH_OF_MAPS);
//     __type(key, u32);
//     __uint(max_entries, 512);
//     __uint(pinning, LIBBPF_PIN_BY_NAME);
//     __array(values, struct each_flow_target);
// } flow_target_map SEC(".maps");

#endif /* __LD_FLOW_H__ */