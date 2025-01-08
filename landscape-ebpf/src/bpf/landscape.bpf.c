
#include "vmlinux.h"

#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#include "landscape.h"
#include "nat.h"

const volatile u8 LOG_LEVEL = BPF_LOG_LEVEL_DEBUG;
char LICENSE[] SEC("license") = "Dual BSD/GPL";

#undef BPF_LOG_LEVEL
#undef BPF_LOG_TOPIC
#define BPF_LOG_LEVEL LOG_LEVEL

#define ETH_PPP_DIS bpf_htons(0x8863)
#define ETH_PPP bpf_htons(0x8864)

#define ETH_PPP_IPV4 bpf_htons(0x0021)
#define ETH_PPP_IPV6 bpf_htons(0x0057)

#define ETH_IPV4 bpf_htons(0x0800) /* ETH IPV4 packet */
#define ETH_IPV6 bpf_htons(0x86DD) /* ETH IPv6 packet */

struct __attribute__((__packed__)) pppoe_header {
    u8 version;
    u8 code;
    u16 session_id;
    u16 length;
    u16 protocol;
};

SEC("tc")
/// @brief  进行出口包的 nat
/// @param skb
/// @return
int modify_egress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "modify_egress"

    bpf_log_info("Egress  - Protocol: %x \\n", bpf_htons(skb->protocol));
    bpf_log_info("egress %u", skb->ifindex);

    // void *data_end = (void *)(long)skb->data_end;
    // void *data = (void *)(long)skb->data;
    // int pkt_sz = data_end - data;
    // struct ethhdr *eth = (struct ethhdr *)(data);
    // if ((void *)(eth + 1) > data_end) {
    //     return TC_ACT_SHOT;
    // }

    // if (eth->h_proto != ETH_IPV4) {
    //     return TC_ACT_UNSPEC;
    // }

    // int result = bpf_skb_adjust_room(skb, 8, BPF_ADJ_ROOM_MAC, 0);
    // if (result) {
    //     bpf_log_info("modify error %d", result);
    //     return TC_ACT_SHOT;
    // } else {
    //     bpf_log_info("modify success");
    // }

    // data_end = (void *)(long)skb->data_end;
    // data = (void *)(long)skb->data;
    // u16 l2_proto = ETH_PPP;
    // struct pppoe_header pppoe = {
    //     .version = 0x11,
    //     .code = 0x00,
    //     .session_id = bpf_htons(0xce05),
    //     .length = bpf_htons(pkt_sz - 14 + 2),
    //     .protocol = bpf_htons(0x0021),
    // };

    // bpf_skb_store_bytes(skb, 12, &l2_proto, sizeof(u16), 0);
    // bpf_skb_store_bytes(skb, sizeof(struct ethhdr), &pppoe, sizeof(struct pppoe_header), 0);
    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}

SEC("tc")
/// 进行入口包的 nat
int ingress_nat(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "ingress_nat"
    // u16 protocol;
    // u32 protocol_offset = offsetof(struct __sk_buff, family);
    // bpf_core_read(&protocol, sizeof(u16), skb->family);
    void *data = (void *)(long)skb->data;
    void *data_end = (void *)(long)skb->data_end;

    struct ethhdr *eth = data;

    if (eth + 1 > data_end) return TC_ACT_UNSPEC;

    bpf_printk("Got packet");
    if (eth->h_proto != bpf_htons(ETH_PPP)) return TC_ACT_OK;

    struct pppoe_header *last = eth + 1;
    if ((void *)(last + 1) > data_end) return TC_ACT_OK;
    bpf_printk("Got PPP packet %d", last->length);
    // u32 family = skb->family;
    // bpf_log_info("Ingress - offsetof: %u", protocol_offset);
    // bpf_log_info("Ingress tc_index %u", skb->tc_index);
    // bpf_log_info("Ingress ifindex %u", skb->ifindex);
    // bpf_log_info("Ingress - no ip: %x", bpf_htons(skb->protocol));
    // if (skb->protocol == bpf_htons(ETH_P_IP)) {
    //     // u32 address = &skb->family;
    //     if (eth->h_proto != bpf_htons(ETH_P_IP)) {
    //         bpf_log_info("Ingress - ip: %x", eth->h_dest);
    //         bpf_log_info("Ingress - ip: %x", eth->h_source);
    //         bpf_log_info("Ingress - ip: %x", eth->h_proto);
    //     }
    // } else {
    // }

    // if (skb->family == bpf_htons(2)) {
    //     bpf_log_info("Ingress - sss  family: ");
    // };

    // bpf_log_info("ingress_nat %u", skb->ifindex);
    // struct map_binding_value test = {.to_port = 16};
    // struct nat_conn_key conn_key = {.ifindex = 10};
    // bpf_map_update_elem(&map_binding, &conn_key, &test, BPF_ANY);
    // bpf_log_info("ingress_nat1 %u", &test);

    // struct map_binding_value *test2 = bpf_map_lookup_elem(&map_binding, &conn_key);
    // if (test2) {
    //     bpf_log_info("ingress_nat2 %u", test2);
    // }
    return TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}

SEC("xdp")
int xdp_pass(struct xdp_md *ctx) {
#define BPF_LOG_TOPIC "xdp_pass"
    void *data = (void *)(long)ctx->data;
    void *data_end = (void *)(long)ctx->data_end;
    int pkt_sz = data_end - data;

    struct ethhdr *eth = (struct ethhdr *)(data);
    if ((void *)(eth + 1) > data_end) {
        return XDP_PASS;
    }

    bpf_log_info("ingress_ifindex: %u", ctx->ingress_ifindex);
    // bpf_log_info("egress_ifindex: %u", ctx->egress_ifindex);

    if (eth->h_proto != ETH_PPP) {
        return XDP_PASS;
    }

    u8 src_mac[6];
    u8 dst_mac[6];
    memcpy(src_mac, eth->h_source, 6);
    memcpy(dst_mac, eth->h_dest, 6);
    // 打印 MAC 地址
    bpf_log_info("before MAC: %02x:%02x:%02x:%02x:%02x:%02x", eth->h_source[0], eth->h_source[1],
                 eth->h_source[2], eth->h_source[3], eth->h_source[4], eth->h_source[5]);

    bpf_log_info("before packet size: %d", pkt_sz);
    int result = bpf_xdp_adjust_head(ctx, 8);
    // bpf_printk("bpf_xdp_adjust_head result %d", result);

    data = (void *)(long)ctx->data;
    data_end = (void *)(long)ctx->data_end;
    int after_pkt_sz = data_end - data;
    bpf_log_info("after packet size: %d", after_pkt_sz);

    eth = (struct ethhdr *)(data);
    if ((void *)(eth + 1) > data_end) {
        return XDP_DROP;
    }
    memcpy(eth->h_source, src_mac, 6);
    memcpy(eth->h_dest, dst_mac, 6);
    eth->h_proto = ETH_IPV4;
    bpf_log_info("after old eth MAC: %02x:%02x:%02x:%02x:%02x:%02x", eth->h_source[0],
                 eth->h_source[1], eth->h_source[2], eth->h_source[3], eth->h_source[4],
                 eth->h_source[5]);

    return XDP_PASS;
#undef BPF_LOG_TOPIC
}

SEC("tc")
int mark_egress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "<< mark_egress"

    struct ethhdr *eth;
    if (VALIDATE_READ_DATA(skb, &eth, 0, sizeof(*eth))) {
        return TC_ACT_UNSPEC;
    }

    bpf_log_info("h_proto: %x", bpf_ntohs(eth->h_proto));

    return TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}

SEC("tc")
int mark_ingress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC ">> mark_ingress"
    struct ethhdr *eth;
    if (VALIDATE_READ_DATA(skb, &eth, 0, sizeof(*eth))) {
        return TC_ACT_UNSPEC;
    }

    bpf_log_info("h_proto: %x", bpf_ntohs(eth->h_proto));

    return TC_ACT_UNSPEC;
#undef BPF_LOG_TOPIC
}