import { ServiceStatus } from "@/lib/services";
import type {
  IPV6RAServiceConfig,
  IPv6NAInfo,
  DHCPv6OfferInfo,
} from "@landscape-router/types/api/schemas";
import {
  getAllIcmpv6raStatus,
  getIfaceIcmpv6Config,
  handleIfaceIcmpv6,
  deleteAndStopIfaceIcmpv6,
  getAllIcmpv6raAssignedIps,
  getAllDhcpv6Assigned,
  getDhcpv6AssignedByIfaceName,
} from "@landscape-router/types/api/icmpv6-ra/icmpv6-ra";

// IPv6NAInfo is now directly imported from generated types

export async function get_all_icmpv6ra_status(): Promise<
  Map<string, ServiceStatus>
> {
  const data = await getAllIcmpv6raStatus();
  const map = new Map<string, ServiceStatus>();
  for (const [key, value] of Object.entries(data)) {
    map.set(key, value as ServiceStatus);
  }
  return map;
}

export async function get_iface_icmpv6ra_config(
  iface_name: string,
): Promise<IPV6RAServiceConfig> {
  return await getIfaceIcmpv6Config(iface_name);
}

export async function update_icmpv6ra_config(
  icmpv6ra_config: IPV6RAServiceConfig,
): Promise<void> {
  await handleIfaceIcmpv6(icmpv6ra_config);
}

export async function stop_and_del_iface_icmpv6ra(name: string): Promise<void> {
  await deleteAndStopIfaceIcmpv6(name);
}

export async function get_icmpra_assigned_ips(): Promise<
  Map<string, IPv6NAInfo | null>
> {
  const data = await getAllIcmpv6raAssignedIps();
  const map = new Map<string, IPv6NAInfo | null>();
  for (const [key, value] of Object.entries(data)) {
    map.set(key, value as IPv6NAInfo);
  }
  return map;
}

export async function get_all_dhcpv6_assigned(): Promise<
  Map<string, DHCPv6OfferInfo | null>
> {
  const data = await getAllDhcpv6Assigned();
  const map = new Map<string, DHCPv6OfferInfo | null>();
  for (const [key, value] of Object.entries(data)) {
    map.set(key, value as DHCPv6OfferInfo);
  }
  return map;
}

export async function get_dhcpv6_assigned_by_iface(
  iface_name: string,
): Promise<DHCPv6OfferInfo | null> {
  const data = await getDhcpv6AssignedByIfaceName(iface_name);
  return (data as DHCPv6OfferInfo) ?? null;
}

export type { IPv6NAInfo, DHCPv6OfferInfo };
