import api from ".";
import { ServiceStatus } from "@/lib/services";
import axiosService from ".";
import { IPV6RAServiceConfig } from "landscape-types/common/ra";
import { IPv6NAInfo } from "landscape-types/common/ipv6_ra_server";

export async function get_all_icmpv6ra_status(): Promise<
  Map<string, ServiceStatus>
> {
  let data = await axiosService.get(`services/icmpv6ra/status`);
  let map = new Map<string, ServiceStatus>();
  for (const [key, value] of Object.entries(data.data)) {
    map.set(key, new ServiceStatus(value as any));
  }
  return map;
}

export async function get_iface_icmpv6ra_config(
  iface_name: string
): Promise<IPV6RAServiceConfig> {
  let data = await axiosService.get(`services/icmpv6ra/${iface_name}`);
  console.log(data.data);
  return data.data;
}

export async function update_icmpv6ra_config(
  icmpv6ra_config: IPV6RAServiceConfig
): Promise<void> {
  let data = await axiosService.post(`services/icmpv6ra`, {
    ...icmpv6ra_config,
  });
  console.log(data.data);
  return data.data;
}

export async function stop_and_del_iface_icmpv6ra(name: string): Promise<void> {
  return axiosService.delete(`services/icmpv6ra/${name}`);
}

export async function get_icmpra_assigned_ips(): Promise<
  Map<string, IPv6NAInfo | null>
> {
  let data = await axiosService.get(`services/icmpv6ra/assigned_ips`);
  let map = new Map<string, IPv6NAInfo | null>();
  for (const [key, value] of Object.entries(data.data)) {
    map.set(key, value as IPv6NAInfo);
  }
  return map;
}
