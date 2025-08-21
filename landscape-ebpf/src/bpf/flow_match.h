#ifndef __LD_FLOW_MATCH_H__
#define __LD_FLOW_MATCH_H__
#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include "packet_def.h"
#include "landscape.h"

struct flow_match_key {
    // 源 IP 地址
    union u_inet_addr src_addr;
    // vlan id
    u32 vlan_tci;
    // tos value
    u8 tos;
    // IP 协议: IPv4 Ipv6
    u8 l3_protocol;
    // IP 层协议: TCP / UDP
    u8 l4_protocol;
    u8 _pad;
};


struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, struct flow_match_key);
    __type(value, u32);
    __uint(max_entries, 65536);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} flow_match_map SEC(".maps");

#endif /* __LD_FLOW_MATCH_H__ */