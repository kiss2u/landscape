#ifndef LD_NAT_H
#define LD_NAT_H
#include "vmlinux.h"
#include "landscape_log.h"
#include "packet_def.h"
#include "land_nat_common.h"

#define GRESS_MASK (1 << 0)

#define COPY_ADDR_FROM(t, s) (__builtin_memcpy((t), (s), sizeof(t)))
static __always_inline void inet_addr_set_ip(union u_inet_addr *addr, __be32 ip) { addr->ip = ip; }
static __always_inline bool inet_addr_equal(const union u_inet_addr *a,
                                            const union u_inet_addr *b) {
    return a->ip == b->ip;
}

/// @brief  解析的 ip 数据包载体
struct ip_packet_info {
    u8 _pad;
    // ip 报文承载的协议类型: TCP / UDP / ICMP
    u8 ip_protocol;
    // 数据包的处理类型 (例如, 非链接, SYN FIN)
    u8 pkt_type;
    // 是否还有分片
    u8 fragment_type;
    // 分片偏移量
    u16 fragment_off;
    // 当前分片 id
    u16 fragment_id;
    // l3 的负载偏移位置 当为 0 时表示没有 ip 的负载 也就是没有 TCP ICMP UDP 头部信息
    // 为 0 表示为 IP 的分片
    int l4_payload_offset;
    // icmp 错误时 icmp payload 的负载位置
    // 不为 0 表示 这个是 icmp 错误 包
    int icmp_error_payload_offset;

    struct inet_pair pair_ip;
};

static __always_inline int is_broadcast_ip(const struct ip_packet_info *pkt) {
    bool is_dst_ipv4_broadcast = false;
    bool is_src_ipv4_broadcast = false;

    __be32 dst = pkt->pair_ip.dst_addr.ip;
    __be32 src = pkt->pair_ip.src_addr.ip;

    if (dst == bpf_htonl(0xffffffff) || dst == 0) {
        is_dst_ipv4_broadcast = true;
    }

    if (src == bpf_htonl(0xffffffff) || src == 0) {
        is_src_ipv4_broadcast = true;
    }

    if (is_dst_ipv4_broadcast || is_src_ipv4_broadcast) {
        return TC_ACT_UNSPEC;
    } else {
        return TC_ACT_OK;
    }
}

#endif /* LD_NAT_H */
