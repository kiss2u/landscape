#ifndef __LD_PACKET_SCANNER_H__
#define __LD_PACKET_SCANNER_H__

#include "vmlinux.h"
#include <bpf/bpf_endian.h>
#include "landscape_log.h"
#include "landscape.h"
#include "pkg_def.h"

#define LD_IP_MF bpf_htons(0x2000)     /* Flag: "More Fragments"	*/
#define LD_IP_OFFSET bpf_htons(0x1FFF) /* "Fragment Offset" part	*/

// RFC 8200 要求支持至少 6 个扩展头
#define LD_MAX_IPV6_EXT_NUM 6

// size limit 5 u32
struct packet_offset_info {
    u8 icmp_error_l3_protocol;
    u8 icmp_error_l4_protocol;
    u16 status;

    u8 pkt_type;
    /// LANDSCAPE_IPV4_TYPE | LANDSCAPE_IPV6_TYPE
    u8 l3_protocol;
    u8 l4_protocol;
    u8 fragment_type;

    u16 fragment_off;
    u16 fragment_id;

    // TCP / UDP / ICMP
    u16 l4_offset;
    u16 l3_offset_when_scan;

    // ICMP err msg offset ( IPv4/v6 )
    // l4_offset + fix ICMP HDR LEN, maybe can store other info
    u16 icmp_error_l3_offset;
    // ICMP err msg offset ( TCP / UDP )
    u16 icmp_error_inner_l4_offset;
};

struct packet_info {
    struct packet_offset_info offset;
    struct inet_pair ip_pair;
};

#define PACKET_OFFSET_INFO_TO_CB(skb, info)                                                        \
    do {                                                                                           \
        (skb)->cb[0] = ((u32)(info)->icmp_error_l3_protocol << 24) |                               \
                       ((u32)(info)->icmp_error_l4_protocol << 16) | ((u32)(info)->status);        \
        (skb)->cb[1] = ((u32)(info)->pkt_type << 24) | ((u32)(info)->l3_protocol << 16) |          \
                       ((u32)(info)->l4_protocol << 8) | ((u32)(info)->fragment_type);             \
        (skb)->cb[2] = ((u32)(info)->fragment_off << 16) | ((u32)(info)->fragment_id);             \
        (skb)->cb[3] = ((u32)(info)->l4_offset << 16) | ((u32)(info)->l3_offset_when_scan);        \
        (skb)->cb[4] =                                                                             \
            ((u32)(info)->icmp_error_l3_offset << 16) | ((u32)(info)->icmp_error_inner_l4_offset); \
    } while (0)

#define CB_TO_PACKET_OFFSET_INFO(skb, info)                                                        \
    do {                                                                                           \
        (info)->icmp_error_l3_protocol = ((skb)->cb[0] >> 24) & 0xff;                              \
        (info)->icmp_error_l4_protocol = ((skb)->cb[0] >> 16) & 0xff;                              \
        (info)->status = (skb)->cb[0] & 0xffff;                                                    \
        (info)->pkt_type = ((skb)->cb[1] >> 24) & 0xff;                                            \
        (info)->l3_protocol = ((skb)->cb[1] >> 16) & 0xff;                                         \
        (info)->l4_protocol = ((skb)->cb[1] >> 8) & 0xff;                                          \
        (info)->fragment_type = (skb)->cb[1] & 0xff;                                               \
        (info)->fragment_off = ((skb)->cb[2] >> 16) & 0xffff;                                      \
        (info)->fragment_id = (skb)->cb[2] & 0xffff;                                               \
        (info)->l4_offset = ((skb)->cb[3] >> 16) & 0xffff;                                         \
        (info)->l3_offset_when_scan = (skb)->cb[3] & 0xffff;                                       \
        (info)->icmp_error_l3_offset = ((skb)->cb[4] >> 16) & 0xffff;                              \
        (info)->icmp_error_inner_l4_offset = (skb)->cb[4] & 0xffff;                                \
    } while (0)

static __always_inline void restore_offset_from_cb(struct __sk_buff *skb,
                                                   struct packet_offset_info *offset,
                                                   u32 current_offset) {
    u32 l3_base_offset = 0;
    CB_TO_PACKET_OFFSET_INFO(skb, offset);

    if (offset->l3_offset_when_scan == current_offset) {
        return;
    }
    if (offset->l3_offset_when_scan > current_offset) {
        l3_base_offset = offset->l3_offset_when_scan - current_offset;
        offset->l3_offset_when_scan -= l3_base_offset;
        offset->l4_offset -= l3_base_offset;
        offset->icmp_error_l3_offset -= l3_base_offset;
        offset->icmp_error_inner_l4_offset -= l3_base_offset;
    } else if (offset->l3_offset_when_scan < current_offset) {
        l3_base_offset = current_offset - offset->l3_offset_when_scan;
        offset->l3_offset_when_scan += l3_base_offset;
        offset->l4_offset += l3_base_offset;
        offset->icmp_error_l3_offset += l3_base_offset;
        offset->icmp_error_inner_l4_offset += l3_base_offset;
    }
}

static __always_inline bool is_offset_cached(struct __sk_buff *skb) {
    return (skb->cb[0] & 0xffff) == 1;
}

struct ip_scanner_ctx {
    u8 l4_protocol;
    u8 fragment_type;
    u16 fragment_off;
    u16 fragment_id;
    u16 l4_offset;
};

enum land_scan_result {
    LD_SCAN_OK = 0,
    LD_SCAN_ERR = 2,
    LD_SCAN_UNSPEC = -1,
};

enum land_frag_type {
    // 还有分片
    // offect 且 more 被设置
    LD_MORE_F,
    // 结束分片
    // offect 的值不为 0
    LD_END_F,
    // 没有分片
    LD_NOT_F
};

union u_ld_ip {
    __be32 all[4];
    __be32 ip;
    __be32 ip6[4];
    u8 bits[16];
};

static __always_inline bool ld_ip_addr_equal(const union u_ld_ip *a, const union u_ld_ip *b) {
    return a->all[0] == b->all[0] && a->all[1] == b->all[1] && a->all[2] == b->all[2] &&
           a->all[3] == b->all[3];
}

static __always_inline int scan_ipv4(struct __sk_buff *skb, struct ip_scanner_ctx *scanner_ctx) {
#define BPF_LOG_TOPIC "scan_ipv4"

    struct iphdr *iph;
    if (VALIDATE_READ_DATA(skb, &iph, scanner_ctx->l4_offset, sizeof(struct iphdr))) {
        return LD_SCAN_ERR;
    }

    if (iph->version != 4) {
        return LD_SCAN_ERR;
    }

    scanner_ctx->fragment_off = (bpf_ntohs(iph->frag_off) & LD_IP_OFFSET) << 3;
    if (iph->frag_off & LD_IP_MF) {
        scanner_ctx->fragment_type = LD_MORE_F;
    } else if (scanner_ctx->fragment_off) {
        scanner_ctx->fragment_type = LD_END_F;
    } else {
        scanner_ctx->fragment_type = LD_NOT_F;
    }

    scanner_ctx->fragment_id = bpf_ntohs(iph->id);
    scanner_ctx->l4_protocol = iph->protocol;
    scanner_ctx->l4_offset += (iph->ihl * 4);

    return LD_SCAN_OK;
#undef BPF_LOG_TOPIC
}

static __always_inline int scan_ipv6(struct __sk_buff *skb, struct ip_scanner_ctx *scanner_ctx) {
#define BPF_LOG_TOPIC "scan_ipv6"

    struct ipv6hdr *ip6h;
    if (VALIDATE_READ_DATA(skb, &ip6h, scanner_ctx->l4_offset, sizeof(*ip6h))) {
        return LD_SCAN_ERR;
    }

    if (ip6h->version != 6) {
        return LD_SCAN_ERR;
    }

    int payload_relative_pos = sizeof(struct ipv6hdr) + scanner_ctx->l4_offset;
    u32 frag_hdr_off = 0;
    u8 nexthdr = ip6h->nexthdr;

    struct ipv6_opt_hdr *opthdr;
    struct frag_hdr *frag_hdr;

    for (int i = 0; i < LD_MAX_IPV6_EXT_NUM; i++) {
        switch (nexthdr) {
        case NEXTHDR_AUTH:
            // Just passthrough IPSec packet
            return TC_ACT_UNSPEC;
        case NEXTHDR_FRAGMENT:
            frag_hdr_off = payload_relative_pos;
        case NEXTHDR_HOP:
        case NEXTHDR_ROUTING:
        case NEXTHDR_DEST: {
            if (VALIDATE_READ_DATA(skb, &opthdr, payload_relative_pos, sizeof(*opthdr))) {
                return TC_ACT_SHOT;
            }
            payload_relative_pos += (opthdr->hdrlen + 1) * 8;
            nexthdr = opthdr->nexthdr;
            break;
        }
        default:
            goto found_upper_layer;
        }
    }

    switch (nexthdr) {
    case NEXTHDR_TCP:
    case NEXTHDR_UDP:
    case NEXTHDR_ICMP:
        goto found_upper_layer;
    default:
        return LD_SCAN_ERR;
    }

found_upper_layer:
    if (frag_hdr_off) {
        if (VALIDATE_READ_DATA(skb, &frag_hdr, frag_hdr_off, sizeof(*frag_hdr))) {
            return TC_ACT_SHOT;
        }
        scanner_ctx->fragment_id = bpf_ntohl(frag_hdr->identification);
        scanner_ctx->fragment_off = bpf_ntohs(frag_hdr->frag_off & bpf_htons(IPV6_FRAG_OFFSET));

        if (frag_hdr->frag_off & bpf_htons(IPV6_FRAG_MF)) {
            scanner_ctx->fragment_type = LD_MORE_F;
        } else if (scanner_ctx->fragment_off) {
            scanner_ctx->fragment_type = LD_END_F;
        } else {
            scanner_ctx->fragment_type = LD_NOT_F;
        }
    } else {
        scanner_ctx->fragment_type = LD_NOT_F;
    }

    scanner_ctx->l4_protocol = nexthdr;
    scanner_ctx->l4_offset = payload_relative_pos;

    return LD_SCAN_OK;
#undef BPF_LOG_TOPIC
}

static __always_inline int icmp_msg_type(struct icmphdr *icmph) {
    switch (icmph->type) {
    case ICMP_DEST_UNREACH:
    case ICMP_TIME_EXCEEDED:
    case ICMP_PARAMETERPROB:
        return ICMP_ERROR_MSG;
    case ICMP_ECHOREPLY:
    case ICMP_ECHO:
    case ICMP_TIMESTAMP:
    case ICMP_TIMESTAMPREPLY:
        return ICMP_QUERY_MSG;
    }
    return ICMP_ACT_UNSPEC;
}

static __always_inline int icmp6_msg_type(struct icmp6hdr *icmp6h) {
    switch (icmp6h->icmp6_type) {
    case ICMPV6_DEST_UNREACH:
    case ICMPV6_PKT_TOOBIG:
    case ICMPV6_TIME_EXCEED:
    case ICMPV6_PARAMPROB:
        return ICMP_ERROR_MSG;
    case ICMPV6_ECHO_REQUEST:
    case ICMPV6_ECHO_REPLY:
        return ICMP_QUERY_MSG;
    }
    return ICMP_ACT_UNSPEC;
}

static __always_inline int scan_packet(struct __sk_buff *skb, u32 current_l3_offset,
                                       struct packet_offset_info *offset_info) {
#define BPF_LOG_TOPIC "scan_packet"

    bool is_ipv4;

    if (current_l3_offset != 0) {
        struct ethhdr *eth;
        if (VALIDATE_READ_DATA(skb, &eth, 0, sizeof(*eth))) {
            return LD_SCAN_ERR;
        }

        if (eth->h_proto == ETH_IPV4) {
            offset_info->l3_protocol = LANDSCAPE_IPV4_TYPE;
            is_ipv4 = true;
        } else if (eth->h_proto == ETH_IPV6) {
            offset_info->l3_protocol = LANDSCAPE_IPV6_TYPE;
            is_ipv4 = false;
        } else {
            return LD_SCAN_UNSPEC;
        }
    } else {
        u8 *p_version;
        if (VALIDATE_READ_DATA(skb, &p_version, 0, sizeof(*p_version))) {
            return LD_SCAN_ERR;
        }
        u8 ip_version = (*p_version) >> 4;
        if (ip_version == 4) {
            offset_info->l3_protocol = LANDSCAPE_IPV4_TYPE;
            is_ipv4 = true;
        } else if (ip_version == 6) {
            offset_info->l3_protocol = LANDSCAPE_IPV6_TYPE;
            is_ipv4 = false;
        } else {
            return LD_SCAN_UNSPEC;
        }
    }

    struct ip_scanner_ctx ctx = {0};
    offset_info->l3_offset_when_scan = current_l3_offset;
    ctx.l4_offset = current_l3_offset;
    if (is_ipv4) {
        if (scan_ipv4(skb, &ctx)) {
            bpf_log_info("scan ip v4 err");
            return LD_SCAN_ERR;
        }
    } else {
        if (scan_ipv6(skb, &ctx)) {
            bpf_log_info("scan ip v6 err");
            return LD_SCAN_ERR;
        }
    }

    __builtin_memcpy(&offset_info->l4_protocol, &ctx, sizeof(struct ip_scanner_ctx));
    // offset_info->l4_protocol = ctx.l4_protocol;
    // offset_info->fragment_type = ctx.fragment_type;
    // offset_info->fragment_off = ctx.fragment_off;
    // offset_info->fragment_id = ctx.fragment_id;
    // offset_info->l4_offset = ctx.l4_offset;

    if (offset_info->fragment_type != LD_NOT_F && offset_info->fragment_off != 0) {
        // 不是第一个数据包， 整个都是 payload
        // 因为没有头部信息, 所以 需要进行查询已有的 track 记录
        offset_info->l4_offset = 0;
        return LD_SCAN_OK;
    }

    __builtin_memset(&ctx, 0, sizeof(ctx));
    if (offset_info->l4_protocol == IPPROTO_ICMP) {
        struct icmphdr *icmph;
        if (VALIDATE_READ_DATA(skb, &icmph, offset_info->l4_offset, sizeof(struct icmphdr))) {
            bpf_log_info("icmphdr error, offset_info->l4_offset: %u", offset_info->l4_offset);
            return LD_SCAN_ERR;
        }
        switch (icmp_msg_type(icmph)) {
        case ICMP_ERROR_MSG: {
            offset_info->icmp_error_l3_offset = offset_info->l4_offset + ICMP_HDR_LEN;
            barrier_var(offset_info->icmp_error_l3_offset);
            ctx.l4_offset = offset_info->icmp_error_l3_offset;
            if (scan_ipv4(skb, &ctx)) {
                bpf_log_info("scan icmp inner ipv4 error: %u", ctx.l4_offset);
                return LD_SCAN_ERR;
            }

            if (ctx.fragment_off) {
                // icmp 不处理分片导致的 icmp 错误
                bpf_log_error("could not handle icmp with fragment");
                return LD_SCAN_ERR;
            }

            offset_info->icmp_error_inner_l4_offset = ctx.l4_offset;
            offset_info->icmp_error_l3_protocol = LANDSCAPE_IPV4_TYPE;
            offset_info->icmp_error_l4_protocol = ctx.l4_protocol;

            u32 *temp_addr;
            u32 dst_ip_val, icmp_src_ip_val;
            if (VALIDATE_READ_DATA(skb, &temp_addr,
                                   offset_info->l3_offset_when_scan + offsetof(struct iphdr, daddr),
                                   sizeof(u32))) {
                return TC_ACT_SHOT;
            }
            dst_ip_val = *temp_addr;
            if (VALIDATE_READ_DATA(skb, &temp_addr,
                                   offset_info->icmp_error_l3_offset +
                                       offsetof(struct iphdr, saddr),
                                   sizeof(u32))) {
                return TC_ACT_SHOT;
            }
            icmp_src_ip_val = *temp_addr;

            if (dst_ip_val != icmp_src_ip_val) {
                bpf_log_error("IP destination address does not match source "
                              "address inside ICMP error message");
                return LD_SCAN_ERR;
            }
            break;
        }
        case ICMP_QUERY_MSG: {
            break;
        }
        case ICMP_ACT_UNSPEC:
            return LD_SCAN_UNSPEC;
        default:
            bpf_log_error("icmp shot");
            return LD_SCAN_ERR;
        }
    } else if (offset_info->l4_protocol == IPPROTO_ICMPV6) {
        struct icmp6hdr *icmph;
        if (VALIDATE_READ_DATA(skb, &icmph, offset_info->l4_offset, sizeof(struct icmp6hdr))) {
            return TC_ACT_SHOT;
        }

        switch (icmp6_msg_type(icmph)) {
        case ICMP_ERROR_MSG: {
            offset_info->icmp_error_l3_offset = offset_info->l4_offset + ICMP_HDR_LEN;
            ctx.l4_offset = offset_info->icmp_error_l3_offset;
            if (scan_ipv6(skb, &ctx)) {
                bpf_log_info("scan icmpv6 inner ipv6 error: %u", ctx.l4_offset);
                return LD_SCAN_ERR;
            }

            if (ctx.fragment_off) {
                // icmp 不处理分片导致的 icmp 错误
                return LD_SCAN_ERR;
            }

            offset_info->icmp_error_inner_l4_offset = ctx.l4_offset;
            offset_info->icmp_error_l3_protocol = LANDSCAPE_IPV6_TYPE;
            offset_info->icmp_error_l4_protocol = ctx.l4_protocol;

            union u_ld_ip *temp_addr;
            union u_ld_ip dst_ip_val, icmp_src_ip_val;

            if (VALIDATE_READ_DATA(skb, &temp_addr,
                                   offset_info->l3_offset_when_scan +
                                       offsetof(struct ipv6hdr, daddr),
                                   sizeof(union u_ld_ip))) {
                return TC_ACT_SHOT;
            }
            COPY_ADDR_FROM(dst_ip_val.all, temp_addr->all);
            if (VALIDATE_READ_DATA(skb, &temp_addr,
                                   offset_info->icmp_error_l3_offset +
                                       offsetof(struct ipv6hdr, saddr),
                                   sizeof(union u_ld_ip))) {
                return TC_ACT_SHOT;
            }
            COPY_ADDR_FROM(icmp_src_ip_val.all, temp_addr->all);

            if (!ld_ip_addr_equal(&dst_ip_val, &icmp_src_ip_val)) {
                bpf_log_error("IP destination address does not match source "
                              "address inside ICMP error message");
                return LD_SCAN_ERR;
            }
            break;
        }
        case ICMP_QUERY_MSG: {
            break;
        }
        case ICMP_ACT_UNSPEC:
            return LD_SCAN_UNSPEC;
        default:
            bpf_log_error("icmp shot");
            return LD_SCAN_ERR;
        }
    }

    return LD_SCAN_OK;
#undef BPF_LOG_TOPIC
}

static __always_inline bool is_icmp_error_pkt(const struct packet_offset_info *offset) {
    return offset->icmp_error_l3_offset > 0 && offset->icmp_error_inner_l4_offset > 0;
}

#endif /* __LD_PACKET_SCANNER_H__ */