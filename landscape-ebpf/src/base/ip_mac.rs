use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::mem::MaybeUninit;
use std::net::Ipv4Addr;
use std::time::Duration;

use landscape_common::net::MacAddr;
use libbpf_rs::skel::{OpenSkel, SkelBuilder};
use libbpf_rs::{ErrorKind, MapCore, MapFlags};
use tokio::sync::oneshot;

pub(crate) mod neigh_update {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bpf_rs/neigh_update.skel.rs"));
}

use neigh_update::*;

use crate::base::ip_mac::neigh_update::types::mac_key_v4;
use crate::base::ip_mac::neigh_update::types::mac_value_v4;
use crate::{bpf_error::LdEbpfResult, landscape::pin_and_reuse_map, MAP_PATHS};

const ARP_SYNC_INTERVAL_SECS: u64 = 10;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ArpSyncStats {
    deleted: usize,
    upserted: usize,
}

pub fn neigh_update(mut service_status: oneshot::Receiver<()>) -> LdEbpfResult<()> {
    let mut open_object = MaybeUninit::zeroed();
    let builder = NeighUpdateSkelBuilder::default();
    let mut open_skel =
        crate::bpf_ctx!(builder.open(&mut open_object), "neigh_update open skeleton failed")?;

    crate::bpf_ctx!(
        pin_and_reuse_map(&mut open_skel.maps.ip_mac_v4, &MAP_PATHS.ip_mac_v4),
        "neigh_update prepare ip_mac_v4 failed"
    )?;
    crate::bpf_ctx!(
        pin_and_reuse_map(&mut open_skel.maps.ip_mac_v6, &MAP_PATHS.ip_mac_v6),
        "neigh_update prepare ip_mac_v6 failed"
    )?;

    let skel = crate::bpf_ctx!(open_skel.load(), "neigh_update load skeleton failed")?;
    let kprobe_neigh_update = skel.progs.kprobe_neigh_update;

    let _link = match kprobe_neigh_update.attach_kprobe(false, "neigh_update") {
        Ok(link) => Some(link),
        Err(e) => {
            // Keep the periodic `/proc/net/arp` sync alive even when this kernel
            // does not expose the neigh_update kprobe symbol.
            tracing::warn!(
                "failed to attach neigh_update kprobe, falling back to periodic ARP sync only: {e}"
            );
            None
        }
    };

    'm: loop {
        tracing::info!("sync current arpv4 info");
        sync_arp_table_to_ebpf_map();

        for _ in 0..ARP_SYNC_INTERVAL_SECS {
            if let Ok(_) | Err(oneshot::error::TryRecvError::Closed) = service_status.try_recv() {
                tracing::info!("neigh_update service stopping...");
                break 'm;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    Ok(())
}

pub fn sync_arp_table_to_ebpf_map() {
    let entries = match parse_arp_full_info() {
        Ok(entries) => entries,
        Err(e) => {
            tracing::error!("read neigh error, skip current arp sync: {e}");
            return;
        }
    };

    let ip_mac_v4 = match libbpf_rs::MapHandle::from_pinned_path(&MAP_PATHS.ip_mac_v4) {
        Ok(map) => map,
        Err(e) => {
            tracing::error!("open pinned ip_mac_v4 map error, skip current arp sync: {e}");
            return;
        }
    };

    match reconcile_arp_entries_in_map(&ip_mac_v4, &entries) {
        Ok(stats) => {
            tracing::debug!(
                "sync arpv4 info finished: deleted={}, upserted={}",
                stats.deleted,
                stats.upserted
            );
        }
        Err(e) => {
            tracing::error!("reconcile ip_mac_v4 map error: {e}");
        }
    }
}

fn reconcile_arp_entries_in_map<T>(
    map: &T,
    entries: &[(mac_key_v4, mac_value_v4)],
) -> libbpf_rs::Result<ArpSyncStats>
where
    T: MapCore,
{
    let desired_addrs: HashSet<u32> = entries.iter().map(|(key, _)| key.addr).collect();
    let mut stale_keys = Vec::new();

    for raw_key in map.keys() {
        let raw_key = read_unaligned::<mac_key_v4>(&raw_key).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("decode ip_mac_v4 key failed: invalid key size {}", raw_key.len()),
            )
        })?;

        if !desired_addrs.contains(&raw_key.addr) {
            let mut stale_key = mac_key_v4::default();
            stale_key.addr = raw_key.addr;
            stale_keys.push(stale_key);
        }
    }

    for key in &stale_keys {
        if let Err(e) = map.delete(unsafe { plain::as_bytes(key) }) {
            if e.kind() != ErrorKind::NotFound {
                return Err(e);
            }
        }
    }

    if !entries.is_empty() {
        let (keys, values) = build_batch_buffers(entries);
        map.update_batch(&keys, &values, entries.len() as u32, MapFlags::ANY, MapFlags::ANY)?;
    }

    Ok(ArpSyncStats { deleted: stale_keys.len(), upserted: entries.len() })
}

fn build_batch_buffers(entries: &[(mac_key_v4, mac_value_v4)]) -> (Vec<u8>, Vec<u8>) {
    let mut keys = Vec::with_capacity(entries.len() * std::mem::size_of::<mac_key_v4>());
    let mut values = Vec::with_capacity(entries.len() * std::mem::size_of::<mac_value_v4>());

    for (key, value) in entries {
        keys.extend_from_slice(unsafe { plain::as_bytes(key) });
        values.extend_from_slice(unsafe { plain::as_bytes(value) });
    }

    (keys, values)
}

fn read_unaligned<T>(bytes: &[u8]) -> Option<T>
where
    T: Copy,
{
    if bytes.len() != std::mem::size_of::<T>() {
        return None;
    }

    Some(unsafe { std::ptr::read_unaligned(bytes.as_ptr().cast::<T>()) })
}

fn parse_arp_full_info() -> Result<Vec<(mac_key_v4, mac_value_v4)>, std::io::Error> {
    let file = fs::File::open("/proc/net/arp")?;
    let reader = BufReader::new(file);
    parse_arp_entries(reader)
}

fn parse_arp_entries<R>(reader: R) -> Result<Vec<(mac_key_v4, mac_value_v4)>, std::io::Error>
where
    R: BufRead,
{
    let mut results = Vec::new();

    // 缓存网卡名对应的本地 MAC，避免重复读取同一个网卡的文件
    let mut dev_mac_cache = std::collections::HashMap::new();

    for line in reader.lines().skip(1) {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 6 {
            continue;
        }

        let ip_str = fields[0];
        let neighbor_mac_str = fields[3];
        let dev_name = fields[5];

        if neighbor_mac_str == "00:00:00:00:00:00" {
            continue;
        }

        let ip_addr = ip_str.parse().unwrap_or(Ipv4Addr::UNSPECIFIED);
        if ip_addr.is_unspecified() {
            continue;
        }

        let neighbor_mac = MacAddr::from_str(neighbor_mac_str).unwrap_or(MacAddr::zero());

        let ifindex = match nix::net::if_::if_nametoindex(dev_name) {
            Ok(idx) => idx,
            Err(_) => continue,
        };

        let device_mac = if let Some(mac) = dev_mac_cache.get(dev_name) {
            *mac
        } else {
            let mac = get_device_mac(dev_name).unwrap_or(MacAddr::zero());
            dev_mac_cache.insert(dev_name.to_string(), mac);
            mac
        };

        let mut key = mac_key_v4::default();
        let mut value = mac_value_v4::default();

        key.addr = ip_addr.to_bits().to_be();
        value.ifindex = ifindex;
        value.proto = 0x0008;
        value.mac = neighbor_mac.octets();
        value.dev_mac = device_mac.octets();

        results.push((key, value));
    }
    Ok(results)
}

fn get_device_mac(dev_name: &str) -> Option<MacAddr> {
    let path = format!("/sys/class/net/{}/address", dev_name);
    let mac_str = fs::read_to_string(path).ok()?;
    MacAddr::from_str(&mac_str)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use libbpf_rs::{libbpf_sys, MapHandle, MapType};

    use super::*;

    fn create_test_ip_mac_map() -> MapHandle {
        #[allow(clippy::needless_update)]
        let opts = libbpf_sys::bpf_map_create_opts {
            sz: std::mem::size_of::<libbpf_sys::bpf_map_create_opts>() as libbpf_sys::size_t,
            ..Default::default()
        };

        MapHandle::create(
            MapType::Hash,
            Option::<&str>::None,
            std::mem::size_of::<mac_key_v4>() as u32,
            std::mem::size_of::<mac_value_v4>() as u32,
            128,
            &opts,
        )
        .expect("create ip_mac test map")
    }

    fn make_entry(ip: &str, mac: &str, dev_mac: &str, ifindex: u32) -> (mac_key_v4, mac_value_v4) {
        let mut key = mac_key_v4::default();
        key.addr = Ipv4Addr::from_str(ip).unwrap().to_bits().to_be();

        let mut value = mac_value_v4::default();
        value.ifindex = ifindex;
        value.mac = MacAddr::from_str(mac).unwrap().octets();
        value.dev_mac = MacAddr::from_str(dev_mac).unwrap().octets();
        value.proto = 0x0008;

        (key, value)
    }

    fn insert_entry<T>(map: &T, entry: &(mac_key_v4, mac_value_v4))
    where
        T: MapCore,
    {
        map.update(
            unsafe { plain::as_bytes(&entry.0) },
            unsafe { plain::as_bytes(&entry.1) },
            MapFlags::ANY,
        )
        .expect("insert ip_mac entry");
    }

    fn lookup_entry<T>(map: &T, ip: &str) -> Option<mac_value_v4>
    where
        T: MapCore,
    {
        let mut key = mac_key_v4::default();
        key.addr = Ipv4Addr::from_str(ip).unwrap().to_bits().to_be();

        map.lookup(unsafe { plain::as_bytes(&key) }, MapFlags::ANY)
            .expect("lookup ip_mac entry")
            .map(|value| read_unaligned::<mac_value_v4>(&value).expect("decode ip_mac entry"))
    }

    #[test]
    fn reconcile_updates_existing_ip_mac_entry() {
        let map = create_test_ip_mac_map();
        let original = make_entry("10.0.0.8", "02:11:22:33:44:55", "02:aa:bb:cc:dd:ee", 7);
        insert_entry(&map, &original);

        let updated = make_entry("10.0.0.8", "02:66:77:88:99:aa", "02:aa:bb:cc:dd:ee", 9);
        let stats = reconcile_arp_entries_in_map(&map, &[updated]).expect("reconcile ip_mac map");

        assert_eq!(stats.deleted, 0);
        assert_eq!(stats.upserted, 1);
        let stored = lookup_entry(&map, "10.0.0.8").expect("entry missing after update");
        assert_eq!(stored.ifindex, 9);
        assert_eq!(stored.mac, updated.1.mac);
    }

    #[test]
    fn reconcile_deletes_stale_ip_mac_entries() {
        let map = create_test_ip_mac_map();
        let keep = make_entry("10.0.0.8", "02:11:22:33:44:55", "02:aa:bb:cc:dd:ee", 7);
        let stale = make_entry("10.0.0.9", "02:66:77:88:99:aa", "02:aa:bb:cc:dd:ef", 8);
        insert_entry(&map, &keep);
        insert_entry(&map, &stale);

        let stats = reconcile_arp_entries_in_map(&map, &[keep]).expect("reconcile ip_mac map");

        assert_eq!(stats.deleted, 1);
        assert_eq!(lookup_entry(&map, "10.0.0.8").unwrap().mac, keep.1.mac);
        assert!(lookup_entry(&map, "10.0.0.9").is_none());
    }

    #[test]
    fn reconcile_clears_map_when_arp_snapshot_is_empty() {
        let map = create_test_ip_mac_map();
        let first = make_entry("10.0.0.8", "02:11:22:33:44:55", "02:aa:bb:cc:dd:ee", 7);
        let second = make_entry("10.0.0.9", "02:66:77:88:99:aa", "02:aa:bb:cc:dd:ef", 8);
        insert_entry(&map, &first);
        insert_entry(&map, &second);

        let stats = reconcile_arp_entries_in_map(&map, &[]).expect("reconcile ip_mac map");

        assert_eq!(stats.deleted, 2);
        assert_eq!(stats.upserted, 0);
        assert_eq!(map.keys().count(), 0);
    }
}
