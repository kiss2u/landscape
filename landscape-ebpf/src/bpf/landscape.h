#ifndef __LD_LANDSCAPE_H__
#define __LD_LANDSCAPE_H__
#include "vmlinux.h"
#include "landscape_log.h"

#define TC_ACT_UNSPEC (-1)
#define TC_ACT_OK 0
#define TC_ACT_SHOT 2
#define TC_ACT_PIPE 3
#define TC_ACT_REDIRECT 7

#define BPF_LOOP_RET_CONTINUE 0
#define BPF_LOOP_RET_BREAK 1

#define ETH_P_8021Q 0x8100
#define LAND_REDIRECT_NETNS_VLAN_ID 0x1d

#define ETH_IPV4 bpf_htons(0x0800) /* ETH IPV4 packet */
#define ETH_IPV6 bpf_htons(0x86DD) /* ETH IPv6 packet */
#define ETH_ARP bpf_htons(0x0806)  /* ETH ARP packet */

#define AF_INET 2
#define AF_INET6 10

// L4 proto number
#define IPPROTO_ICMPV6 58

// EGRESS MARK
#define OK_MARK 0
#define DIRECT_MARK 1
#define DROP_MARK 2
#define REDIRECT_MARK 3
#define SYMMETRIC_NAT 4
#define REDIRECT_NETNS_MARK 5

#define ACTION_MASK 0x00FF
#define INDEX_MASK 0xFF00

#define FLOW_KEEP_GOING 0
#define FLOW_DIRECT 1
#define FLOW_DROP 2
#define FLOW_REDIRECT 3
#define FLOW_ALLOW_REUSE 4

#define FLOW_FROM_UNKNOW 0
#define FLOW_FROM_HOST 1
#define FLOW_FROM_LAN 2
#define FLOW_FROM_WAN 4

#define FLOW_SOURCE_MASK 0xFF000000
#define FLOW_ACTION_MASK 0x00007F00
#define FLOW_ALLOW_REUSE_PORT_MASK 0x00008000
#define FLOW_ID_MASK 0x000000FF

// 替换 FLOW_ID_MASK 对应的 0~7 位
static __always_inline u32 replace_flow_id(u32 original, u8 new_id) {
    original &= ~FLOW_ID_MASK;         // 清除原来的 ID 部分
    original |= ((u32)new_id & 0xFF);  // 设置新的 ID 部分
    return original;
}

// 替换 FLOW_ACTION_MASK 对应的 8~14 位
static __always_inline u32 replace_flow_action(u32 original, u8 new_action) {
    original &= ~FLOW_ACTION_MASK;              // 清除原来的 Action 部分
    original |= ((u32)new_action & 0x7F) << 8;  // 只取低 7 bit，写入 8~14 位
    return original;
}

// 替换 FLOW_ALLOW_REUSE_PORT_MASK 对应的第 15 位
static __always_inline u32 set_flow_allow_reuse_port(u32 original, bool allow) {
    original &= ~FLOW_ALLOW_REUSE_PORT_MASK;  // 清除原来的标志位
    if (allow) {
        original |= FLOW_ALLOW_REUSE_PORT_MASK;  // 设置为 1
    }
    return original;
}

// 替换 FLOW_SOURCE_MASK 对应的 24~31 位
static __always_inline u32 replace_flow_source(u32 original, u8 new_source) {
    original &= ~FLOW_SOURCE_MASK;               // 清除原来的 Source 部分
    original |= ((u32)new_source & 0xFF) << 24;  // 设置新的 Source 部分
    return original;
}

static __always_inline u8 get_flow_id(u32 original) { return (original & FLOW_ID_MASK); }

// 获取 action
static __always_inline u8 get_flow_action(u32 original) {
    return (original & FLOW_ACTION_MASK) >> 8;  // 返回 0–127
}

// 获取 reuse port 标志
static __always_inline bool get_flow_allow_reuse_port(u32 original) {
    return (original & FLOW_ALLOW_REUSE_PORT_MASK) != 0;
}

static __always_inline u8 get_flow_source(u32 original) {
    return (original & FLOW_SOURCE_MASK) >> 24;
}

// INGRESS MARK
#define INGRESS_NO_MARK 0
#define INGRESS_STATIC_MARK 1

#define INGRESS_CACHE_MASK 0x000000FF

// 替换 INGRESS_CACHE_MASK 对应的 0~7 位
static __always_inline u32 replace_cache_mask(u32 original, u8 new_mark) {
    original &= ~INGRESS_CACHE_MASK;
    original |= ((u32)new_mark & 0xFF);
    return original;
}

static __always_inline u8 get_cache_mask(u32 original) { return (original & INGRESS_CACHE_MASK); }

#define PRINT_MAC_ADDR(mac)                                                                        \
    bpf_log_info("mac: %02x:%02x:%02x:%02x:%02x:%02x", (mac)[0], (mac)[1], (mac)[2], (mac)[3],     \
                 (mac)[4], (mac)[5])


#ifndef likely
#define likely(x) __builtin_expect(!!(x), 1)
#endif

#ifndef unlikely
#define unlikely(x) __builtin_expect(!!(x), 0)
#endif

#define MAX_OFFSET 20480

static __always_inline int _validate_read(struct __sk_buff *skb, void **hdr_, u32 offset, u32 len) {
    if (unlikely(offset > MAX_OFFSET || len > 256 || offset + len > MAX_OFFSET)) return 1;

    void *data = (void *)(long)skb->data;
    void *data_end = (void *)(long)skb->data_end;
    void *hdr = data + offset;

    barrier_var(hdr);
    if (likely(hdr + len <= data_end)) {
        *hdr_ = hdr;
        return 0;
    }

    if (bpf_skb_pull_data(skb, offset + len)) return 1;

    data = (void *)(long)skb->data;
    hdr = data + offset;

    if (hdr + len > (void *)(long)skb->data_end) return 1;

    *hdr_ = hdr;
    return 0;
}

#define VALIDATE_READ_DATA(skb, hdr, off, len) (_validate_read(skb, (void **)hdr, off, len))

struct ipv4_lpm_key {
    __u32 prefixlen;
    __be32 addr;
};

struct ipv6_lpm_key {
    __u32 prefixlen;
    struct in6_addr addr;
};

struct ipv4_mark_action {
    __u32 mark;
};

static int prepend_dummy_mac(struct __sk_buff *skb) {
    u8 mac[] = {0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 0x08, 0x00};

    if (bpf_skb_change_head(skb, 14, 0)) return -1;

    if (bpf_skb_store_bytes(skb, 0, mac, sizeof(mac), 0)) return -1;

    return 0;
}

static int store_mac_v4(struct __sk_buff *skb, u8 *dst_mac, u8 *src_mac) {
    u8 mac[14];

    __builtin_memcpy(mac, dst_mac, 6);
    __builtin_memcpy(mac + 6, src_mac, 6);
    
    mac[12] = 0x08;
    mac[13] = 0x00;

    if (bpf_skb_store_bytes(skb, 0, mac, sizeof(mac), 0)) return -1;

    return 0;
}

static int store_mac_v6(struct __sk_buff *skb, u8 *dst_mac, u8 *src_mac) {
    u8 mac[14];

    __builtin_memcpy(mac, dst_mac, 6);
    __builtin_memcpy(mac + 6, src_mac, 6);
    
    mac[12] = 0x86;
    mac[13] = 0xdd;

    if (bpf_skb_store_bytes(skb, 0, mac, sizeof(mac), 0)) return -1;

    return 0;
}


// TEMP
// ICMPv4 消息类型
enum {
    ICMP_ERROR_MSG,
    ICMP_QUERY_MSG,
    ICMP_ACT_UNSPEC,
    ICMP_ACT_SHOT,
};

union u_inet_addr {
    __be32 all[4];
    __be32 ip;
    __be32 ip6[4];
    u8 bits[16];
};

// only for ipv6
union u_inet6_addr {
    __be32 all[4];
    __be32 ip;
    __be32 ip6[4];
    u8 bytes[16];
};

struct inet_pair {
    union u_inet_addr src_addr;
    union u_inet_addr dst_addr;
    __be16 src_port;
    __be16 dst_port;
};

enum fragment_type {
    // 还有分片
    // offect 且 more 被设置
    MORE_F,
    // 结束分片
    // offect 的值不为 0
    END_F,
    // 没有分片
    NOT_F
};

// 数据包所属的连接类型
enum {
    // 无连接
    PKT_CONNLESS,
    //
    PKT_TCP_DATA,
    PKT_TCP_SYN,
    PKT_TCP_RST,
    PKT_TCP_FIN,
    PKT_TCP_ACK,
};

/// 作为 fragment 缓存的 key
struct fragment_cache_key {
    u8 _pad[3];
    u8 l4proto;
    u32 id;
    union u_inet_addr saddr;
    union u_inet_addr daddr;
};

struct fragment_cache_value {
    u16 sport;
    u16 dport;
};

static __always_inline int is_broadcast_mac(struct __sk_buff *skb) {
    u8 *mac;

    if (VALIDATE_READ_DATA(skb, &mac, 0, 6)) {
        return TC_ACT_UNSPEC;
    }

    // 判断是否是广播地址 ff:ff:ff:ff:ff:ff
    bool is_broadcast = mac[0] == 0xff && mac[1] == 0xff && mac[2] == 0xff && mac[3] == 0xff &&
                        mac[4] == 0xff && mac[5] == 0xff;

    bool is_ipv6_broadcast = mac[0] == 0x33 && mac[1] == 0x33;

    if (unlikely(is_broadcast || is_ipv6_broadcast)) {
        return TC_ACT_UNSPEC;
    } else {
        return TC_ACT_OK;
    }
}


static __always_inline int is_broadcast_ip4(__be32 dst) {
    // 255.255.255.255 or 0.0.0.0 (network byte order)
    if (dst == 0xffffffff || dst == 0) {
        return TC_ACT_UNSPEC;
    }
    return TC_ACT_OK;
}


static __always_inline int is_broadcast_ip6(const u8 *bytes) {
    bool is_ipv6_broadcast = false;
    bool is_ipv6_locallink = false;

    __u8 first_byte = bytes[0];

    // IPv6 multicast ff00::/8
    if (first_byte == 0xff) {
        is_ipv6_broadcast = true;
    }

    // IPv6 link-local fe80::/10
    if (first_byte == 0xfe) {
        __u8 second_byte = bytes[1];
        if ((second_byte & 0xc0) == 0x80) {  // top 2 bits == 10
            is_ipv6_locallink = true;
        }
    }

    if (unlikely(is_ipv6_broadcast || is_ipv6_locallink)) {
        return TC_ACT_UNSPEC;
    } else {
        return TC_ACT_OK;
    }
}
#endif /* __LD_LANDSCAPE_H__ */