#ifndef __LD_IP_NEIGH_H__
#define __LD_IP_NEIGH_H__
#include <bpf/bpf_helpers.h>
#include "landscape.h"

struct mac_key_v4 {
    __be32 addr;
};

struct mac_value_v4 {
    u8 mac[6];
};

struct  {
    __uint(type, BPF_MAP_TYPE_LRU_HASH);
    __type(key, struct mac_key_v4);
    __type(value, struct mac_value_v4);
    __uint(max_entries, 4096);
} ip_mac_v4 SEC(".maps");

struct mac_key_v6 {
    union u_inet6_addr addr;
};

struct mac_value_v6 {
    u8 mac[6];
};

struct  {
    __uint(type, BPF_MAP_TYPE_LRU_HASH);
    __type(key, struct mac_key_v6);
    __type(value, struct mac_value_v6);
    __uint(max_entries, 4096);
} ip_mac_v6 SEC(".maps");


#endif /* __LD_IP_NEIGH_H__ */