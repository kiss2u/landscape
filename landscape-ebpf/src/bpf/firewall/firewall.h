#ifndef __LD_FIREWALL_H__
#define __LD_FIREWALL_H__
#include <bpf/bpf_endian.h>

#include <vmlinux.h>
#include "../landscape_log.h"
#include "../landscape.h"
#include "../pkg_def.h"

#define IPV4_FIREWALL_EGRESS_PROG_INDEX 0
#define IPV4_FIREWALL_INGRESS_PROG_INDEX 0
#define IPV6_FIREWALL_EGRESS_PROG_INDEX 1
#define IPV6_FIREWALL_INGRESS_PROG_INDEX 1

/// @brief 持有解析的 IP 信息
struct ip_context {
    // ip 报文承载的协议类型: TCP / UDP / ICMP
    u8 ip_protocol;
    // 数据包的处理类型 (例如, 非链接, SYN FIN)
    u8 pkt_type;
    // ICMP Type
    u8 icmp_type;
    // 分片类型，例如 NOT_F、MORE_F、END_F
    u8 fragment_type;
    // 分片偏移量
    u16 fragment_off;
    // 当前分片 id 标识符
    u16 fragment_id;
    // IPv4 键值对
    struct inet_pair pair_ip;
};

/// @brief 数据包解析上下文
struct packet_context {
    struct ip_context ip_hdr;
    // l4 的负载偏移位置 当为 0 时表示没有 ip 的负载 也就是没有 TCP ICMP UDP 头部信息
    // 为 -1 表示为 IP 的分片
    int l4_payload_offset;
    // icmp 错误时指向 l4 的负载起始位置
    // 不为 0 表示 这个是 icmp 错误 包
    int icmp_error_payload_offset;
};

#endif /* __LD_FIREWALL_H__ */
