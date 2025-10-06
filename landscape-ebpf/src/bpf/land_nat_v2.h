#ifndef LD_NAT_V2_H
#define LD_NAT_V2_H
#include "vmlinux.h"
#include "landscape_log.h"
#include "pkg_scanner.h"
#include "pkg_fragment.h"
#include "land_nat_common.h"

///
struct ip_packet_info_v2 {
    struct packet_offset_info offset;
    struct inet_pair pair_ip;
};

#endif /* LD_NAT_V2_H */
