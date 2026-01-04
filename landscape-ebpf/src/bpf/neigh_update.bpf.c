#include "vmlinux.h"
#include <bpf/bpf_helpers.h>

#include "landscape.h"
#include "ip_neigh.h"

char LICENSE[] SEC("license") = "GPL";

struct trace_event_raw_neigh_event {
    __u16 common_type;
    __u8 common_flags;
    __u8 common_preempt_count;
    __s32 common_pid;

    __u32 family;
    __u32 dev;  // __data_loc char[]
    __u8 lladdr[32];
    __u8 lladdr_len;
    __u8 flags;
    __u8 nud_state;
    __u8 type;
    __u8 dead;
    __s32 refcnt;
    __u8 primary_key4[4];
    __u8 primary_key6[16];
    unsigned long confirmed;
    unsigned long updated;
    unsigned long used;
    __u8 new_lladdr[32];
    __u8 new_state;
    __u32 update_flags;
    __u32 pid;
};

#define NEIGH_UPDATE_F_OVERRIDE 0x01
#define NEIGH_UPDATE_F_WEAK_OVERRIDE 0x02

#define NUD_VALID (0x02 | 0x04 | 0x40 | 0x80)
#define NUD_FAILED 0x20
#define AF_INET 2
#define AF_INET6 10

SEC("tracepoint/neigh/neigh_update")
int trace_neigh_update(struct trace_event_raw_neigh_event *ctx) {
    u64 update_flag = 0;
    if (ctx->update_flags & NEIGH_UPDATE_F_OVERRIDE) {
        update_flag = BPF_ANY;
    } else if (ctx->update_flags & NEIGH_UPDATE_F_WEAK_OVERRIDE) {
        update_flag = BPF_NOEXIST;
    } else {
        return 0;
    }

    if (ctx->family == AF_INET) {
        struct mac_key_v4 key = {};
        bpf_probe_read_kernel(&key.addr, sizeof(key.addr), ctx->primary_key4);

        if (key.addr == 0) {
            bpf_printk("SKIP: IP is 0.0.0.0 | state: %d -> %d", ctx->nud_state, ctx->new_state);
            return 0;
        }

        if (ctx->new_state & NUD_VALID) {
            struct mac_value_v4 value = {};
            bpf_probe_read_kernel(value.mac, 6, ctx->new_lladdr);

            if (value.mac[0] == 0 && value.mac[1] == 0 && value.mac[2] == 0) {
                bpf_probe_read_kernel(value.mac, 6, ctx->lladdr);
            }

            if (value.mac[0] != 0 || value.mac[1] != 0 || value.mac[5] != 0) {
                bpf_map_update_elem(&ip_mac_v4, &key, &value, update_flag);
                // bpf_printk("Update IP:%pI4", &key.addr);
                // PRINT_MAC_ADDR(value.mac);
            }
        } else if (ctx->new_state == NUD_FAILED) {
            bpf_map_delete_elem(&ip_mac_v4, &key);
            // bpf_printk("MAP_DELETE: ip=%pI4 due to NUD_FAILED", &key.addr);
        }

        // bpf_printk("neigh state change: %d -> %d", ctx->nud_state, ctx->new_state);
    } else if (ctx->family == AF_INET6) {
        struct mac_key_v6 key = {};
        bpf_probe_read_kernel(&key.addr, sizeof(key.addr), ctx->primary_key6);

        if (is_broadcast_ip6(key.addr.bytes)) {
            bpf_printk("SKIP: IP is %pI6 | state: %d -> %d", key.addr.bytes, ctx->nud_state,
                       ctx->new_state);
            return 0;
        }

        if (ctx->new_state & NUD_VALID) {
            struct mac_value_v6 value = {};
            bpf_probe_read_kernel(value.mac, 6, ctx->new_lladdr);

            if (value.mac[0] == 0 && value.mac[1] == 0 && value.mac[2] == 0) {
                bpf_probe_read_kernel(value.mac, 6, ctx->lladdr);
            }

            if (value.mac[0] != 0 || value.mac[1] != 0 || value.mac[5] != 0) {
                bpf_map_update_elem(&ip_mac_v6, &key, &value, update_flag);
                // bpf_printk("Update: IP is %pI6 | state: %d -> %d", key.addr.bytes,
                // ctx->nud_state, ctx->new_state);
                // bpf_printk("IP:%pI6 | Flags:0x%x | %d->%d", key.addr.bytes, ctx->update_flags,
                //            ctx->nud_state, ctx->new_state);
                // PRINT_MAC_ADDR(value.mac);
            }
        } else if (ctx->new_state == NUD_FAILED) {
            bpf_map_delete_elem(&ip_mac_v6, &key);
            // bpf_printk("MAP_DELETE: ip=%pI6 due to NUD_FAILED", &key.addr);
        }

        // bpf_printk("neigh state change: %d -> %d", ctx->nud_state, ctx->new_state);
    }

    return 0;
}
