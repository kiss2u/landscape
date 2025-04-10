#ifndef __LD_FLOW_MARK_SHARE_H__
#define __LD_FLOW_MARK_SHARE_H__
#include <bpf/bpf_helpers.h>
#include "landscape.h"
#include "packet_def.h"
#include "flow.h"

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, struct flow_match_key);
    __type(value, u32);
    __uint(max_entries, 65535);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} flow_match_map SEC(".maps");

#endif /* __LD_FLOW_MARK_SHARE_H__ */