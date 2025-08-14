#include <bpf/bpf_helpers.h>

#include "vmlinux.h"
#include "nat.h"
#include "land_wan_ip.h"

#define STATIC_NAT_MAPPING_CACHE_SIZE 1024 * 64


struct static_nat_mapping_key {
    u32 prefixlen;
    // INGRESS: NAT Mapping Port
    // EGRESS: lan Clinet Port
    u16 port;
    u8 gress;
    // IPv4 / IPv6
    u8 l3_protocol;
    u8 l4_protocol;
    u8 _pad[3];
    // INGRESS:  only use u32 for ifindex match
    // EGRESS: match lan client ip
    union u_inet_addr addr;
};

struct {
    __uint(type, BPF_MAP_TYPE_LPM_TRIE);
    __type(key, struct static_nat_mapping_key);
    __type(value, struct nat_mapping_value);
    __uint(max_entries, STATIC_NAT_MAPPING_CACHE_SIZE);
    __uint(map_flags, BPF_F_NO_PREALLOC);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} static_nat_mappings SEC(".maps");

#define NAT_CREATE_CONN 1
#define NAT_DELETE_CONN 2
struct nat_conn_event {
    union u_inet_addr src_addr;
    union u_inet_addr dst_addr;
    u16 src_port;
    u16 dst_port;
    u64 time;
    u8 l4_proto;
    u8 l3_proto;
    u8 event_type;
    u8 flow_id;
    u8 trace_id;
} __nat_conn_event;

struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 1 << 24);
} nat_conn_events SEC(".maps");
