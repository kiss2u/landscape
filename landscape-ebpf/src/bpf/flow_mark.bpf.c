#include "vmlinux.h"

#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#include "landscape.h"
#include "flow_mark_share.h"

char LICENSE[] SEC("license") = "Dual BSD/GPL";

const volatile int current_eth_net_offset = 14;

SEC("tc/ingress")
int flow_ingress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC ">> flow_ingress"
    struct ethhdr *eth;
    if (VALIDATE_READ_DATA(skb, &eth, 0, sizeof(*eth))) {
        return TC_ACT_UNSPEC;
    }

    skb->mark = 100;

    bpf_log_info("mark: %d", skb->mark);

    return TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}

SEC("tc/egress")
int flow_egress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC ">> flow_egress"
    struct ethhdr *eth;
    if (VALIDATE_READ_DATA(skb, &eth, 0, sizeof(*eth))) {
        return TC_ACT_UNSPEC;
    }

    bpf_log_info("mark: %d", skb->mark);

    return TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}