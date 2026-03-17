#ifndef __LD_NAT_V3_MAPS_H__
#define __LD_NAT_V3_MAPS_H__

#include <bpf/bpf_helpers.h>

#include <vmlinux.h>

#include "../land_nat_common.h"
#include "nat_maps.h"

#define NAT4_V3_TIMER_SIZE NAT_MAPPING_TIMER_SIZE
#define NAT4_V3_PORT_QUEUE_SIZE 65536

struct nat4_port_queue_value_v3 {
    __be16 port;
    u16 last_generation;
};

struct nat_timer_value_v4_v3 {
    u64 server_status;
    u64 client_status;
    u64 status;
    struct bpf_timer timer;
    struct inet4_addr client_addr;
    u16 client_port;
    u8 gress;
    u8 flow_id;
    u64 create_time;
    u64 ingress_bytes;
    u64 ingress_packets;
    u64 egress_bytes;
    u64 egress_packets;
    u32 cpu_id;
    u32 ifindex;
    u16 generation_snapshot;
    u8 is_final_releaser;
    u8 _pad0;
};

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, struct nat_mapping_key_v4);
    __type(value, struct nat_mapping_value_v4_v3);
    __uint(max_entries, NAT_MAPPING_CACHE_SIZE);
} nat4_dyn_map SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, struct nat_timer_key_v4);
    __type(value, struct nat_timer_value_v4_v3);
    __uint(max_entries, NAT4_V3_TIMER_SIZE);
    __uint(map_flags, BPF_F_NO_PREALLOC);
} nat4_mapping_timer_v3 SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_QUEUE);
    __type(value, struct nat4_port_queue_value_v3);
    __uint(max_entries, NAT4_V3_PORT_QUEUE_SIZE);
} nat4_tcp_free_ports_v3 SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_QUEUE);
    __type(value, struct nat4_port_queue_value_v3);
    __uint(max_entries, NAT4_V3_PORT_QUEUE_SIZE);
} nat4_udp_free_ports_v3 SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_QUEUE);
    __type(value, struct nat4_port_queue_value_v3);
    __uint(max_entries, NAT4_V3_PORT_QUEUE_SIZE);
} nat4_icmp_free_ports_v3 SEC(".maps");

#endif /* __LD_NAT_V3_MAPS_H__ */
