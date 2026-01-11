#ifndef LD_IPV4_H
#define LD_IPV4_H
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_endian.h>
#include "vmlinux.h"
#include "../landscape.h"

struct inet4_addr {
    __be32 addr;
};

struct inet4_pair {
    struct inet4_addr src_addr;
    struct inet4_addr dst_addr;
    __be16 src_port;
    __be16 dst_port;
};

static __always_inline bool inet4_addr_equal(const struct inet4_addr *a,
                                            const struct inet4_addr *b) {
    return a->addr == b->addr;
}


#endif /* LD_IPV4_H */
