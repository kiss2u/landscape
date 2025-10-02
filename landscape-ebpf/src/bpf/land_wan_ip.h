#ifndef __LD_WAN_IP_H__
#define __LD_WAN_IP_H__
#include <bpf/bpf_helpers.h>

#include "vmlinux.h"

struct wan_ip_info_key {
    u32 ifindex;
    u8 l3_protocol;
    u8 _pad[3];
};

struct wan_ip_info_value {
    union u_inet_addr addr;
    union u_inet_addr gateway;
    u8 mask;
};


struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, struct wan_ip_info_key);    // index
    __type(value, struct wan_ip_info_value);  // ipv4
    __uint(max_entries, 256);
    __uint(map_flags, BPF_F_NO_PREALLOC);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} wan_ipv4_binding SEC(".maps");

#endif /* __LD_WAN_IP_H__ */