#ifndef __LD_LAN_TC_PIPELINE_H__
#define __LD_LAN_TC_PIPELINE_H__

#include "vmlinux.h"

#include <bpf/bpf_helpers.h>

#include "landscape.h"

#define LAN_INGRESS_STAGE_ROUTE 0
#define LAN_INGRESS_STAGE_COUNT 1

#define LAN_EGRESS_STAGE_ROUTE 0
#define LAN_EGRESS_STAGE_COUNT 1

struct {
    __uint(type, BPF_MAP_TYPE_PROG_ARRAY);
    __uint(max_entries, LAN_INGRESS_STAGE_COUNT);
    __uint(key_size, sizeof(u32));
    __uint(value_size, sizeof(u32));
} lan_ingress_stage_progs SEC(".maps");

struct {
    __uint(type, BPF_MAP_TYPE_PROG_ARRAY);
    __uint(max_entries, LAN_EGRESS_STAGE_COUNT);
    __uint(key_size, sizeof(u32));
    __uint(value_size, sizeof(u32));
} lan_egress_stage_progs SEC(".maps");

static __always_inline int lan_tc_pipeline_tailcall_ingress_from(struct __sk_buff *skb, u32 stage) {
    switch (stage) {
    default:
        bpf_tail_call(skb, &lan_ingress_stage_progs, LAN_INGRESS_STAGE_ROUTE);
        return TC_ACT_UNSPEC;
    case LAN_INGRESS_STAGE_ROUTE:
        return TC_ACT_UNSPEC;
    }
}

static __always_inline int lan_tc_pipeline_tailcall_egress_from(struct __sk_buff *skb, u32 stage) {
    switch (stage) {
    default:
        bpf_tail_call(skb, &lan_egress_stage_progs, LAN_EGRESS_STAGE_ROUTE);
        return TC_ACT_UNSPEC;
    case LAN_EGRESS_STAGE_ROUTE:
        return TC_ACT_UNSPEC;
    }
}

static __always_inline int lan_tc_pipeline_continue_ingress(struct __sk_buff *skb, u32 stage,
                                                            int action) {
    return action == TC_ACT_UNSPEC ? lan_tc_pipeline_tailcall_ingress_from(skb, stage) : action;
}

static __always_inline int lan_tc_pipeline_continue_egress(struct __sk_buff *skb, u32 stage,
                                                           int action) {
    return action == TC_ACT_UNSPEC ? lan_tc_pipeline_tailcall_egress_from(skb, stage) : action;
}

#endif
