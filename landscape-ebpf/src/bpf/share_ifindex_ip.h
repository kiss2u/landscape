#include <bpf/bpf_helpers.h>

#include "vmlinux.h"
#include "nat.h"

#define STATIC_NAT_MAPPING_CACHE_SIZE 1024 * 64
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, u32);    // index
    __type(value, u32);  // ipv4
    __uint(max_entries, 16);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} wan_ipv4_binding SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, struct nat_mapping_key);
    __type(value, struct nat_mapping_value);
    __uint(max_entries, STATIC_NAT_MAPPING_CACHE_SIZE);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} static_nat_mappings SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, u16);
    __type(value, u8);
    __uint(max_entries, STATIC_NAT_MAPPING_CACHE_SIZE);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} nat_expose_ports SEC(".maps");
