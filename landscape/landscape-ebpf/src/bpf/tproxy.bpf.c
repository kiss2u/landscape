#include "vmlinux.h"

#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#include "landscape.h"

const volatile u8 LOG_LEVEL = BPF_LOG_LEVEL_DEBUG;

#undef BPF_LOG_LEVEL
#undef BPF_LOG_TOPIC
#define BPF_LOG_LEVEL LOG_LEVEL

/* Define constants not captured by BTF */
#define BPF_F_CURRENT_NETNS (-1L)
#define TC_ACT_UNSPEC (-1)
#define TC_ACT_OK 0
#define TC_ACT_SHOT 2
#define TC_ACT_PIPE 3
#define ETH_P_IP (0x0800)

#define PACKET_HOST 0
#define PACKET_BROADCAST 1
#define PACKET_MULTICAST 2
#define PACKET_OTHERHOST 3

volatile const __be16 target_port = 0;
volatile const __be32 target_addr = 0;
volatile const __be32 proxy_addr = 0;
volatile const __be16 proxy_port = 0;

/* Fill 'tuple' with L3 info, and attempt to find L4. On fail, return NULL. */
static inline struct bpf_sock_tuple *get_tuple(struct __sk_buff *skb) {
    void *data_end = (void *)(long)skb->data_end;
    void *data = (void *)(long)skb->data;
    struct bpf_sock_tuple *result;
    struct ethhdr *eth;
    __u64 tuple_len;
    __u8 proto = 0;
    __u64 ihl_len;

    eth = (struct ethhdr *)(data);
    if (eth + 1 > data_end) return NULL;

    /* Only support ipv4 */
    if (eth->h_proto != bpf_htons(ETH_P_IP)) return NULL;

    struct iphdr *iph = (struct iphdr *)(data + sizeof(*eth));
    if (iph + 1 > data_end) return NULL;
    if (iph->ihl != 5) /* Options are not supported */
        return NULL;
    ihl_len = iph->ihl * 4;
    proto = iph->protocol;
    result = (struct bpf_sock_tuple *)&iph->saddr;

    /* Only support TCP */
    if (proto != IPPROTO_TCP) return NULL;

    return result;
}

static inline int handle_tcp(struct __sk_buff *skb, struct bpf_sock_tuple *tuple) {
#define BPF_LOG_TOPIC "handle_tcp"
    struct bpf_sock_tuple server = {};
    struct bpf_sock *sk;
    const int zero = 0;
    size_t tuple_len;
    int ret;
    int change_type_err;

    tuple_len = sizeof(tuple->ipv4);
    if ((void *)tuple + tuple_len > (void *)(long)skb->data_end) return TC_ACT_SHOT;

    /* Only proxy packets destined for the target port */
    // if (tuple->ipv4.daddr != target_addr) return TC_ACT_OK;
    if (tuple->ipv4.dport != target_port) return TC_ACT_OK;

    if (tuple->ipv4.sport && tuple->ipv4.dport) {
        bpf_log_info(
            "Source IP: %d.%d.%d.%d, Source Port: %d, Dest IP: %d.%d.%d.%d, Dest Port: %d\n",
            (tuple->ipv4.saddr >> 24) & 0xFF,  // 获取第一个字节
            (tuple->ipv4.saddr >> 16) & 0xFF,  // 获取第二个字节
            (tuple->ipv4.saddr >> 8) & 0xFF,   // 获取第三个字节
            tuple->ipv4.saddr & 0xFF,          // 获取第四个字节
            bpf_ntohs(tuple->ipv4.sport),
            (tuple->ipv4.daddr >> 24) & 0xFF,  // 目标 IP 第一个字节
            (tuple->ipv4.daddr >> 16) & 0xFF,  // 目标 IP 第二个字节
            (tuple->ipv4.daddr >> 8) & 0xFF,   // 目标 IP 第三个字节
            tuple->ipv4.daddr & 0xFF,          // 目标 IP 第四个字节
            bpf_ntohs(tuple->ipv4.dport));
    }

    /* Reuse existing connection if it exists */
    sk = bpf_skc_lookup_tcp(skb, tuple, tuple_len, BPF_F_CURRENT_NETNS, 0);
    if (sk) {
        bpf_log_info("find 1 success: %d", sk);
        if (sk->state != BPF_TCP_LISTEN) goto assign;
        bpf_sk_release(sk);
    }

    /* Lookup port server is listening on */
    server.ipv4.saddr = tuple->ipv4.saddr;
    server.ipv4.daddr = proxy_addr;
    server.ipv4.sport = tuple->ipv4.sport;
    server.ipv4.dport = proxy_port;
    bpf_log_info("Source IP: %d.%d.%d.%d, Source Port: %d, Dest IP: %d.%d.%d.%d, Dest Port: %d\n",
                 (server.ipv4.saddr >> 24) & 0xFF,  // 获取第一个字节
                 (server.ipv4.saddr >> 16) & 0xFF,  // 获取第二个字节
                 (server.ipv4.saddr >> 8) & 0xFF,   // 获取第三个字节
                 server.ipv4.saddr & 0xFF,          // 获取第四个字节
                 bpf_ntohs(server.ipv4.sport),
                 (server.ipv4.daddr >> 24) & 0xFF,  // 目标 IP 第一个字节
                 (server.ipv4.daddr >> 16) & 0xFF,  // 目标 IP 第二个字节
                 (server.ipv4.daddr >> 8) & 0xFF,   // 目标 IP 第三个字节
                 server.ipv4.daddr & 0xFF,          // 目标 IP 第四个字节
                 bpf_ntohs(server.ipv4.dport));
    sk = bpf_skc_lookup_tcp(skb, &server, tuple_len, BPF_F_CURRENT_NETNS, 0);
    if (!sk) return TC_ACT_SHOT;
    if (sk->state != BPF_TCP_LISTEN) {
        bpf_sk_release(sk);
        return TC_ACT_SHOT;
    }

    bpf_log_info("find 2 success: sk=%d", sk);
    bpf_log_info("bound_dev_if=%u", sk->bound_dev_if);
    bpf_log_info("family=%u", sk->family);
    // bpf_log_info("type=%u", sk->type);
    // bpf_log_info("protocol=%u", sk->protocol);
    // bpf_log_info("mark=%u", sk->mark);
    // bpf_log_info("priority=%u", sk->priority);
    bpf_log_info("src_ip4=%u", sk->src_ip4);
    bpf_log_info("src_port=%u", sk->src_port);
    bpf_log_info("dst_port=%u", sk->dst_port);
    bpf_log_info("dst_ip4=%u", sk->dst_ip4);
    bpf_log_info("state=%u", sk->state);
    // bpf_log_info("rx_queue_mapping=%d", sk->rx_queue_mapping);
assign:
    skb->mark = 1;
    change_type_err = bpf_skb_change_type(skb, PACKET_HOST);
    // bpf_log_info("change_type_err %d", change_type_err);
    bpf_log_info("pkt_type %d", skb->pkt_type);
    ret = bpf_sk_assign(skb, sk, 0);
    ret = 0;
    bpf_log_info("ret %d", ret);
    bpf_sk_release(sk);
    return ret;
#undef BPF_LOG_TOPIC
}

SEC("tc")
int tproxy(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "tproxy_ingress"
    struct bpf_sock_tuple *tuple;
    int tuple_len;
    int ret = 0;

    tuple = get_tuple(skb);
    if (!tuple) return TC_ACT_OK;

    ret = handle_tcp(skb, tuple);
    return ret == 0 ? TC_ACT_OK : TC_ACT_SHOT;
#undef BPF_LOG_TOPIC
}

char LICENSE[] SEC("license") = "Dual BSD/GPL";
