#ifndef __LD_FLOW_MARK_SHARE_H__
#define __LD_FLOW_MARK_SHARE_H__
#include <bpf/bpf_helpers.h>
#include "landscape.h"
#include "packet_def.h"

struct flow_match_key {
    // 源 mac 地址
    unsigned char h_source[6];
    // vlan id
    u32 vlan_tci;
    // tos value
    u8 tos;
    u8 _pad;
};

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, struct flow_match_key);
    __type(value, u32);
    __uint(max_entries, 65535);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} flow_match_map SEC(".maps");

#endif /* __LD_FLOW_MARK_SHARE_H__ */