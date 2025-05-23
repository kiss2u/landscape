
#include "vmlinux.h"

#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

#include "landscape.h"

const volatile u8 LOG_LEVEL = BPF_LOG_LEVEL_DEBUG;
char LICENSE[] SEC("license") = "Dual BSD/GPL";

#define ETH_IPV4 bpf_htons(0x0800) /* ETH IPV4 packet */
#define ETH_IPV6 bpf_htons(0x86DD) /* ETH IPv6 packet */

#define PACKET_HOST 0
#define PACKET_BROADCAST 1
#define PACKET_MULTICAST 2
#define PACKET_OTHERHOST 3

#define NF_DROP 0
#define NF_ACCEPT 1

#define AF_INET 2

#define BPF_F_CURRENT_NETNS -1

volatile const u32 outer_ifindex = 0;
volatile const __be32 target_addr = 0;

volatile const __be32 proxy_addr = 0;
volatile const __be16 proxy_port = 0;

const volatile int current_eth_net_offset = 14;

static inline struct bpf_sock_tuple *get_tuple(struct __sk_buff *skb, u16 *l3_protocol,
                                               u8 *l4_protocol, int current_eth_net_offset) {
    void *data_end = (void *)(long)skb->data_end;
    void *data = (void *)(long)skb->data;
    struct bpf_sock_tuple *result;
    struct iphdr *iph;
    __u64 tuple_len;
    __u64 ihl_len;

    bpf_log_info("current_eth_net_offset %u", current_eth_net_offset);
    if (current_eth_net_offset != 0) {
        struct ethhdr *eth;
        eth = (struct ethhdr *)(data);
        if (eth + 1 > data_end) return NULL;

        /* Only support ipv4 */
        *l3_protocol = eth->h_proto;
        if (eth->h_proto != ETH_IPV4) return NULL;

        iph = (struct iphdr *)(data + sizeof(*eth));
        if (iph + 1 > data_end) return NULL;
    } else {
        iph = (struct iphdr *)(data);
        if (iph + 1 > data_end) return NULL;
        if (iph->version == 4) {
            *l3_protocol = ETH_IPV4;
        } else if (iph->version == 6) {
            *l3_protocol = ETH_IPV6;
        } else {
            return NULL;
        }
    }

    if (iph->ihl != 5) /* Options are not supported */
        return NULL;
    ihl_len = iph->ihl * 4;
    *l4_protocol = iph->protocol;
    result = (struct bpf_sock_tuple *)&iph->saddr;

    return result;
}

static inline int handle_tcp(struct __sk_buff *skb, struct bpf_sock_tuple *tuple, u8 *l4_protocol) {
#define BPF_LOG_TOPIC "handle_tcp"
    struct bpf_sock_tuple server = {};
    struct bpf_sock *sk;
    // struct bpf_sock *tcp_sk;
    // struct bpf_sock *udp_sk;
    size_t tuple_len;
    int ret;
    int change_type_err;

    tuple_len = sizeof(tuple->ipv4);
    if ((void *)tuple + tuple_len > (void *)(long)skb->data_end) return TC_ACT_SHOT;

    if (tuple->ipv4.sport && tuple->ipv4.dport) {
        bpf_log_info(
            "Source IP: %d.%d.%d.%d, Source Port: %d, Dest IP: %d.%d.%d.%d, Dest Port: %d\n",
            tuple->ipv4.saddr & 0xFF,          // 获取第四个字节
            (tuple->ipv4.saddr >> 8) & 0xFF,   // 获取第三个字节
            (tuple->ipv4.saddr >> 16) & 0xFF,  // 获取第二个字节
            (tuple->ipv4.saddr >> 24) & 0xFF,  // 获取第一个字节
            bpf_ntohs(tuple->ipv4.sport),
            tuple->ipv4.daddr & 0xFF,          // 目标 IP 第四个字节
            (tuple->ipv4.daddr >> 8) & 0xFF,   // 目标 IP 第三个字节
            (tuple->ipv4.daddr >> 16) & 0xFF,  // 目标 IP 第二个字节
            (tuple->ipv4.daddr >> 24) & 0xFF,  // 目标 IP 第一个字节
            bpf_ntohs(tuple->ipv4.dport));
    }

    /* Reuse existing connection if it exists */
    if (*l4_protocol == IPPROTO_TCP) {
        sk = bpf_skc_lookup_tcp(skb, tuple, tuple_len, BPF_F_CURRENT_NETNS, 0);
        if (sk) {
            bpf_log_info("find 1 success: %u, protocol: tcp", sk);
            bpf_log_info("find 1 state: %u", sk->state);
            // tcp_sk = bpf_sk_lookup_tcp(skb, tuple, tuple_len, BPF_F_CURRENT_NETNS, 0);
            // if (tcp_sk) {
            //     bpf_log_info("find 1 tcp_sk success: %u, protocol: tcp", tcp_sk);
            //     bpf_log_info("find 1 tcp_sk state: %u", tcp_sk->state);
            //     bpf_sk_release(tcp_sk);
            // }
            if (sk->state != BPF_TCP_LISTEN) goto assign;
            bpf_sk_release(sk);
        }
        // } else if (l4_protocol == IPPROTO_UDP) {
        //     sk = bpf_sk_lookup_udp(skb, tuple, tuple_len, BPF_F_CURRENT_NETNS, 0);
        //     if (sk) {
        //         bpf_log_info("find 1 success: %u, protocol: udp", sk);
        //         bpf_log_info("src_ip4: %u", sk->src_ip4);
        //         bpf_log_info("src_port: %u", sk->src_port);
        //         bpf_log_info("dst_port: %u", sk->dst_port);
        //         bpf_log_info("dst_ip4: %u", sk->dst_ip4);
        //         bpf_log_info("find 1 state: %u", sk->state);
        //         goto assign;
        //     }
    }

    /* Lookup port server is listening on */
    server.ipv4.saddr = tuple->ipv4.saddr;
    server.ipv4.daddr = proxy_addr;
    server.ipv4.sport = tuple->ipv4.sport;
    server.ipv4.dport = proxy_port;
    bpf_log_info("Source IP: %d.%d.%d.%d, Source Port: %d, Dest IP: %d.%d.%d.%d, Dest Port: %d\n",
                 server.ipv4.saddr & 0xFF,          // 获取第四个字节
                 (server.ipv4.saddr >> 8) & 0xFF,   // 获取第三个字节
                 (server.ipv4.saddr >> 16) & 0xFF,  // 获取第二个字节
                 (server.ipv4.saddr >> 24) & 0xFF,  // 获取第一个字节
                 bpf_ntohs(server.ipv4.sport),
                 server.ipv4.daddr & 0xFF,          // 目标 IP 第四个字节
                 (server.ipv4.daddr >> 8) & 0xFF,   // 目标 IP 第三个字节
                 (server.ipv4.daddr >> 16) & 0xFF,  // 目标 IP 第二个字节
                 (server.ipv4.daddr >> 24) & 0xFF,  // 目标 IP 第一个字节
                 bpf_ntohs(server.ipv4.dport));
    if (*l4_protocol == IPPROTO_TCP) {
        sk = bpf_skc_lookup_tcp(skb, &server, tuple_len, BPF_F_CURRENT_NETNS, 0);
    } else if (*l4_protocol == IPPROTO_UDP) {
        sk = bpf_sk_lookup_udp(skb, &server, tuple_len, BPF_F_CURRENT_NETNS, 0);
    }
    // sk = bpf_skc_lookup_tcp(skb, &server, tuple_len, BPF_F_CURRENT_NETNS, 0);
    // tcp_sk = bpf_sk_lookup_tcp(skb, &server, tuple_len, BPF_F_CURRENT_NETNS, 0);
    // if (tcp_sk) {
    //     bpf_log_info("find tcp_sk success: sk=%d", tcp_sk);
    //     bpf_sk_release(tcp_sk);
    // }
    // udp_sk = bpf_sk_lookup_udp(skb, &server, tuple_len, BPF_F_CURRENT_NETNS, 0);
    // if (udp_sk) {
    //     bpf_log_info("find udp_sk success: sk=%d", udp_sk);
    //     bpf_sk_release(udp_sk);
    // }

    if (!sk) return TC_ACT_SHOT;
    if (*l4_protocol == IPPROTO_TCP && sk->state != BPF_TCP_LISTEN) {
        bpf_sk_release(sk);
        return TC_ACT_SHOT;
    }

    bpf_log_info("find 2 success: sk:%u, protocol: %d", sk, *l4_protocol);
    // bpf_log_info("bound_dev_if: %u", sk->bound_dev_if);
    // bpf_log_info("family: %u", sk->family);
    // bpf_log_info("type: %u", sk->type);
    // bpf_log_info("protocol: %u", sk->protocol);
    // bpf_log_info("mark: %u", sk->mark);
    // bpf_log_info("priority: %u", sk->priority);
    // bpf_log_info("src_ip4: %u", sk->src_ip4);
    // bpf_log_info("src_port: %u", sk->src_port);
    // bpf_log_info("dst_port: %u", sk->dst_port);
    // bpf_log_info("dst_ip4: %u", sk->dst_ip4);
    // bpf_log_info("state: %u", sk->state);
    // bpf_log_info("rx_queue_mapping=%d", sk->rx_queue_mapping);
assign:
    skb->mark = 1;
    change_type_err = bpf_skb_change_type(skb, PACKET_HOST);
    bpf_log_info("change_type_err %d", change_type_err);
    bpf_log_info("pkt_type %d", skb->pkt_type);
    bpf_log_info("ingress_ifindex %d", skb->ingress_ifindex);
    // bpf_log_info("sk->state %u", sk->state);
    ret = bpf_sk_assign(skb, sk, 0);
    bpf_log_info("bpf_sk_assign ret %d", ret);
    bpf_sk_release(sk);
    return ret;
#undef BPF_LOG_TOPIC
}

SEC("tracepoint/syscalls/sys_enter_setsockopt")
int handle_setsockopt_enter(struct trace_event_raw_sys_enter *ctx) {
    // syscall参数在 ctx->args[] 里：args[0] = fd, args[1] = level, ...
    int fd = ctx->args[0];
    int level = ctx->args[1];
    int opt = ctx->args[2];
    // args[3] 其实是 optval, args[4] 是 optlen, 需要小心处理
    bpf_printk("setsockopt: fd=%d level=%d opt=%d\n", fd, level, opt);
    return 0;
}

// SEC("kprobe/__sys_setsockopt")
// int handle__setsockopt(struct pt_regs *ctx) {
//     int fd = (int)PT_REGS_PARM1(ctx);
//     int level = (int)PT_REGS_PARM2(ctx);
//     int opt = (int)PT_REGS_PARM3(ctx);
//     char *val = (char *)PT_REGS_PARM4(ctx);
//     int vallen = (int)PT_REGS_PARM5(ctx);

//     // 做一些处理
//     bpf_printk("setsockopt: fd=%d level=%d opt=%d\n", fd, level, opt);

//     return 0;
// }

// SEC("sockops")
// int sock_create(struct bpf_sock_ops *skops) {
//     int optname = 19;             // 目标参数
//     int level = 0;                // IPv4 层级
//     int optval = 0;               // 存储结果
//     int optlen = sizeof(optval);  // 参数长度

//     int ret = bpf_getsockopt(skops, level, optname, &optval, optlen);
//     if (ret != 0) {
//         bpf_printk("bpf_getsockopt failed: %d\n", ret);
//         return 0;
//     }

//     if (optval == 1) {
//         bpf_printk("IP_TRANSPARENT is enabled\n");
//     } else {
//         bpf_printk("IP_TRANSPARENT is disabled\n");
//     }

//     switch (skops->op) {
//     // When creating a connection to another host
//     case BPF_SOCK_OPS_TCP_CONNECT_CB:
//         break;
//     // When accepting a connection from another host
//     case BPF_SOCK_OPS_ACTIVE_ESTABLISHED_CB:
//         break;
//     // When the socket is established
//     case BPF_SOCK_OPS_PASSIVE_ESTABLISHED_CB: {
//         // 调用 bpf_getsockopt 读取参数

//         break;
//     }
//     // When reserving space for TCP options header
//     case BPF_SOCK_OPS_HDR_OPT_LEN_CB:
//         break;
//     // When writing TCP options header
//     case BPF_SOCK_OPS_WRITE_HDR_OPT_CB:
//         break;
//     }

//     return 1;
// }

SEC("tc")
int ns_ingress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "ns_inner_ingress"

    u32 vlan_id = skb->vlan_tci;
    if (vlan_id != LAND_REDIRECT_NETNS_VLAN_ID) {
        return TC_ACT_OK;
    }
    bpf_printk("vlan_id: %x", vlan_id);
    bpf_skb_vlan_pop(skb);

    struct bpf_sock_tuple *tuple;
    u16 l3_protocol;
    u8 l4_protocol;
    int ret = 0;

    tuple = get_tuple(skb, &l3_protocol, &l4_protocol, 14);
    if (!tuple) return TC_ACT_OK;

    /* Only support TCP */
    if (l4_protocol != IPPROTO_TCP && l4_protocol != IPPROTO_UDP) {
        bpf_log_info("not support protocol %u", l3_protocol);
        return TC_ACT_OK;
    }

    ret = handle_tcp(skb, tuple, &l4_protocol);
    return ret == 0 ? TC_ACT_OK : TC_ACT_SHOT;
#undef BPF_LOG_TOPIC
}

// SEC("tc")
// int ns_peer_ingress(struct __sk_buff *skb) {
// #define BPF_LOG_TOPIC "ns_outer_ingress"

//     struct bpf_sock_tuple *tuple;
//     tuple = get_tuple(skb);
//     if (!tuple) return TC_ACT_OK;

//     size_t tuple_len;

//     tuple_len = sizeof(tuple->ipv4);
//     if ((void *)tuple + tuple_len > (void *)(long)skb->data_end) return TC_ACT_SHOT;

//     if (tuple->ipv4.sport && tuple->ipv4.dport) {
//         bpf_log_info("Source IP: %d.%d.%d.%d, Source Port: %d, Dest IP: %d.%d.%d.%d, Dest Port:
//         %d",
//                      (tuple->ipv4.saddr >> 24) & 0xFF,  // 获取第一个字节
//                      (tuple->ipv4.saddr >> 16) & 0xFF,  // 获取第二个字节
//                      (tuple->ipv4.saddr >> 8) & 0xFF,   // 获取第三个字节
//                      tuple->ipv4.saddr & 0xFF,          // 获取第四个字节
//                      bpf_ntohs(tuple->ipv4.sport),
//                      (tuple->ipv4.daddr >> 24) & 0xFF,  // 目标 IP 第一个字节
//                      (tuple->ipv4.daddr >> 16) & 0xFF,  // 目标 IP 第二个字节
//                      (tuple->ipv4.daddr >> 8) & 0xFF,   // 目标 IP 第三个字节
//                      tuple->ipv4.daddr & 0xFF,          // 目标 IP 第四个字节
//                      bpf_ntohs(tuple->ipv4.dport));
//     }

//     if (tuple->ipv4.daddr == target_addr) {
//         return bpf_redirect(2, BPF_F_INGRESS);
//     }

//     // bpf_log_info("packet type %d\n", skb->pkt_type);
//     // bpf_log_info("mark %d\n", skb->mark);
//     // struct bpf_fib_lookup fib_params = {};
//     // fib_params.family = AF_INET;
//     // fib_params.tos = 0;
//     // fib_params.l4_protocol = 0;
//     // fib_params.sport = 0;
//     // fib_params.dport = 0;
//     // fib_params.tot_len = tuple_len;
//     // fib_params.ipv4_src = tuple->ipv4.saddr;
//     // fib_params.ipv4_dst = tuple->ipv4.daddr;
//     // fib_params.ifindex = skb->ifindex;

//     // // 调用 bpf_fib_lookup 执行查找
//     // int ret = bpf_fib_lookup(skb, &fib_params, sizeof(fib_params), 0);

//     // if (ret == BPF_FIB_LKUP_RET_SUCCESS) {
//     //     // 设置 MAC 地址
//     //     bpf_log_info("Next hop ifindex: %d egress", fib_params.ifindex);
//     //     bpf_skb_store_bytes(skb, 0, fib_params.dmac, 6, 0);
//     //     bpf_skb_store_bytes(skb, 6, fib_params.smac, 6, 0);
//     //     bpf_skb_change_type(skb, PACKET_OTHERHOST);
//     //     // 使用 bpf_redirect 将数据包发送到下一跳接口
//     //     return bpf_redirect(fib_params.ifindex, 0);
//     // } else {
//     //     bpf_log_info("Next hop fail ret value is: %d\n", ret);
//     //     // 查找失败，放行数据包
//     //     return TC_ACT_OK;
//     // }

//     // fib_params.ipv4_src = tuple->ipv4.daddr;
//     // fib_params.ipv4_dst = tuple->ipv4.saddr;

//     // ret = bpf_fib_lookup(skb, &fib_params, sizeof(fib_params), 0);

//     // if (ret == BPF_FIB_LKUP_RET_SUCCESS) {
//     //     // 设置 MAC 地址
//     //     bpf_log_info("Next hop ifindex: %d ingress", fib_params.ifindex);
//     //     // 使用 bpf_redirect 将数据包发送到下一跳接口
//     //     // return bpf_redirect(fib_params.ifindex, BPF_F_INGRESS);
//     // } else {
//     //     bpf_log_info("Next hop fail ret value is: %d\n", ret);
//     //     // 查找失败，放行数据包
//     //     // return TC_ACT_OK;
//     // }

//     bpf_log_info("Drop packet\n");
//     return TC_ACT_UNSPEC;
// #undef BPF_LOG_TOPIC
// }

SEC("tc")
int wan_egress(struct __sk_buff *skb) {
#define BPF_LOG_TOPIC "wan_egress"
    struct bpf_sock_tuple *tuple;
    u16 l3_protocol;
    u8 l4_protocol;
    int ret;

    tuple = get_tuple(skb, &l3_protocol, &l4_protocol, current_eth_net_offset);
    if (!tuple) {
        bpf_log_info("get_tuple fail");
        return TC_ACT_OK;
    }

    size_t tuple_len;

    tuple_len = sizeof(tuple->ipv4);
    if ((void *)tuple + tuple_len > (void *)(long)skb->data_end) return TC_ACT_SHOT;

    if (tuple->ipv4.daddr != target_addr) return TC_ACT_OK;
    if (current_eth_net_offset == 0) {
        bpf_log_info("current_eth_net_offset is 0 prepend_dummy_mac");
        if (prepend_dummy_mac(skb) != 0) {
            bpf_log_info("prepend_dummy_mac error");
            return TC_ACT_SHOT;
        }
    }
    bpf_skb_vlan_push(skb, ETH_P_8021Q, LAND_REDIRECT_NETNS_VLAN_ID);

    // if (tuple->ipv4.sport && tuple->ipv4.dport) {
    //     bpf_log_info(
    //         "Source IP: %d.%d.%d.%d, Source Port: %d, Dest IP: %d.%d.%d.%d, Dest Port: %d\n",
    //         (tuple->ipv4.saddr >> 24) & 0xFF,  // 获取第一个字节
    //         (tuple->ipv4.saddr >> 16) & 0xFF,  // 获取第二个字节
    //         (tuple->ipv4.saddr >> 8) & 0xFF,   // 获取第三个字节
    //         tuple->ipv4.saddr & 0xFF,          // 获取第四个字节
    //         bpf_ntohs(tuple->ipv4.sport),
    //         (tuple->ipv4.daddr >> 24) & 0xFF,  // 目标 IP 第一个字节
    //         (tuple->ipv4.daddr >> 16) & 0xFF,  // 目标 IP 第二个字节
    //         (tuple->ipv4.daddr >> 8) & 0xFF,   // 目标 IP 第三个字节
    //         tuple->ipv4.daddr & 0xFF,          // 目标 IP 第四个字节
    //         bpf_ntohs(tuple->ipv4.dport));
    // }
    skb->mark = 777;
    ret = bpf_redirect(outer_ifindex, 0);
    bpf_log_info("bpf_redirect result %d", ret);
    return ret;
#undef BPF_LOG_TOPIC
}

SEC("xdp")
int inner_xdp(struct xdp_md *ctx) {
#define BPF_LOG_TOPIC "inner_xdp"
    void *data = (void *)(long)ctx->data;
    void *data_end = (void *)(long)ctx->data_end;
    int pkt_sz = data_end - data;

    bpf_printk("has frame size : %u", pkt_sz);

    struct ethhdr *eth = (struct ethhdr *)(data);
    if ((void *)(eth + 1) > data_end) {
        bpf_log_info("package size smaller then ethhdr");
        return XDP_DROP;
    }

    // if (eth->h_proto = ETH_VLAN) {
    //     bpf_log_info("eth %x", eth->h_proto);
    // }

    return XDP_PASS;
#undef BPF_LOG_TOPIC
}

// SEC("sk_lookup")
// int sk_lookup_test(struct bpf_sk_lookup *ctx) {
// #define BPF_LOG_TOPIC "sk_lookup_test"
//     if (ctx->sk->mark == 1) {
//         bpf_log_info("tttttttttttttttttttttt lookup_test ttttttttttttttttt");
//     }
// #undef BPF_LOG_TOPIC
// }

// SEC("netfilter")
// int netfilter_per_routing(struct bpf_nf_ctx *ctx) {
// #define BPF_LOG_TOPIC "netfilter_per_routing"
//     if (ctx->skb->mark == 1) {
//     bpf_log_info("netfilter pkt_type %d", ctx->skb->pkt_type);
//     }
//     return NF_ACCEPT;
// #undef BPF_LOG_TOPIC
// }

// SEC("netfilter")
// int netfilter_localhost_in(struct bpf_nf_ctx *ctx) {
// #define BPF_LOG_TOPIC "netfilter_localhost_in"
//         if (ctx->skb->mark == 1) {
//     bpf_log_info("netfilter pkt_type %d", ctx->skb->pkt_type);
//     }
//     return NF_ACCEPT;
// #undef BPF_LOG_TOPIC
// }