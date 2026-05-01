#ifndef __LD_ROUTE_PACKET_H__
#define __LD_ROUTE_PACKET_H__

#include <vmlinux.h>

#include <bpf/bpf_endian.h>

#include "../landscape.h"
#include "../pkg_scanner.h"
#include "route_index.h"

static __always_inline int scan_route_packet(struct __sk_buff *skb, u32 current_l3_offset,
                                             struct packet_offset_info *offset_info) {
    return scan_packet_l3(skb, current_l3_offset, offset_info);
}

static __always_inline int read_route_context_v4_from_scan(struct __sk_buff *skb,
                                                           const struct packet_offset_info *offset,
                                                           struct route_context_v4 *context) {
#define BPF_LOG_TOPIC "read_route_context_v4_from_scan"
    if (offset->l3_protocol != LANDSCAPE_IPV4_TYPE) return TC_ACT_UNSPEC;

    struct iphdr *iph;
    if (VALIDATE_READ_DATA(skb, &iph, offset->l3_offset_when_scan, sizeof(*iph))) {
        return TC_ACT_SHOT;
    }

    context->saddr = iph->saddr;
    context->daddr = iph->daddr;
    context->l4_protocol = offset->l4_protocol;
    context->tos = iph->tos;
    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}

static __always_inline int read_route_context_v6_from_scan(struct __sk_buff *skb,
                                                           const struct packet_offset_info *offset,
                                                           struct route_context_v6 *context) {
#define BPF_LOG_TOPIC "read_route_context_v6_from_scan"
    if (offset->l3_protocol != LANDSCAPE_IPV6_TYPE) return TC_ACT_UNSPEC;

    struct ipv6hdr *ip6h;
    if (VALIDATE_READ_DATA(skb, &ip6h, offset->l3_offset_when_scan, sizeof(*ip6h))) {
        return TC_ACT_SHOT;
    }

    COPY_ADDR_FROM(context->saddr.all, ip6h->saddr.in6_u.u6_addr32);
    COPY_ADDR_FROM(context->daddr.all, ip6h->daddr.in6_u.u6_addr32);
    context->l4_protocol = offset->l4_protocol;
    context->tos = 0;
    return TC_ACT_OK;
#undef BPF_LOG_TOPIC
}

static __always_inline int route_should_forward_v4(const struct route_context_v4 *context) {
    return should_not_forward(context->daddr) ? TC_ACT_UNSPEC : TC_ACT_OK;
}

static __always_inline int route_should_forward_v6(const struct route_context_v6 *context) {
    return is_broadcast_ip6(context->daddr.bytes);
}

#endif /* __LD_ROUTE_PACKET_H__ */
