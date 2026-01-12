#ifndef __LD_ROUTE_INDEX_H__
#define __LD_ROUTE_INDEX_H__
#include "vmlinux.h"

#define WAN_CACHE 0
#define LAN_CACHE 1

struct route_context_v4 {
    __be32 saddr;
    __be32 daddr;
    // IP 层协议: TCP / UDP
    u8 l4_protocol;
    // tos value
    u8 tos;
    // TODO
    // u16 dst_port;
    u8 smac[6];
};

struct route_context_v6 {
    union u_inet6_addr saddr;
    union u_inet6_addr daddr;
    // IP 层协议: TCP / UDP
    u8 l4_protocol;
    // tos value
    u8 tos;
    // TODO
    // u16 dst_port;
    u8 smac[6];
};

#endif /* __LD_ROUTE_INDEX_H__ */