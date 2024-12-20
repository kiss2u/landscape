#include "vmlinux.h"
#include "landscape_log.h"

#define TC_ACT_UNSPEC (-1)
#define TC_ACT_OK 0
#define TC_ACT_SHOT 2
#define TC_ACT_PIPE 3

#define BPF_LOOP_RET_CONTINUE 0
#define BPF_LOOP_RET_BREAK 1

#define ETH_IPV4 bpf_htons(0x0800) /* ETH IPV4 packet */
#define ETH_IPV6 bpf_htons(0x86DD) /* ETH IPv6 packet */
#define ETH_ARP bpf_htons(0x0806)  /* ETH ARP packet */

struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __type(key, u32);    // index
    __type(value, u32);  // ipv4
    __uint(max_entries, 16);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} wan_ipv4_binding SEC(".maps");

static __always_inline int _validate_read(struct __sk_buff *skb, void **hdr_, u32 offset, u32 len) {
    u8 *data = (u8 *)(__u64)skb->data;
    u8 *data_end = (u8 *)(__u64)skb->data_end;
    u8 *hdr = (u8 *)(data + offset);

    // ensure hdr pointer is on it's own for validation to work
    barrier_var(hdr);
    if (hdr + len > data_end) {
        if (bpf_skb_pull_data(skb, offset + len)) {
            return 1;
        }

        data = (u8 *)(__u64)skb->data;
        data_end = (u8 *)(__u64)skb->data_end;
        hdr = (u8 *)(data + offset);
        if (hdr + len > data_end) {
            return 1;
        }
    }

    *hdr_ = (void *)hdr;

    return 0;
}

#define VALIDATE_READ_DATA(skb, hdr, off, len) (_validate_read(skb, (void **)hdr, off, len))
