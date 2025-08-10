#ifndef __LD_WAN_IP_H__
#define __LD_WAN_IP_H__
#include <bpf/bpf_helpers.h>

#include "vmlinux.h"

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, u32);    // index
    __type(value, u32);  // ipv4
    __uint(max_entries, 16);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} wan_ipv4_binding SEC(".maps");

#endif /* __LD_WAN_IP_H__ */