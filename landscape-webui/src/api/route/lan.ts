import axiosService from "@/api";
import { ServiceStatus } from "@/lib/services";
import { RouteLanServiceConfig } from "landscape-types/common/route";

export async function get_all_route_lan_status(): Promise<
  Map<string, ServiceStatus>
> {
  let data = await axiosService.get(`services/route_lans/status`);
  let map = new Map<string, ServiceStatus>();
  for (const [key, value] of Object.entries(data.data)) {
    map.set(key, new ServiceStatus(value as any));
  }
  return map;
}

export async function get_route_lans(): Promise<RouteLanServiceConfig[]> {
  let data = await axiosService.get("services/route_lans");
  return data.data;
}

export async function get_route_lan_config(
  id: string
): Promise<RouteLanServiceConfig> {
  let result = await axiosService.get(`services/route_lans/${id}`);
  return result.data;
}

export async function update_route_lans_config(
  config: RouteLanServiceConfig
): Promise<void> {
  await axiosService.post(`services/route_lans`, config);
}

export async function del_route_lans(iface_name: string): Promise<void> {
  await axiosService.delete(`services/route_lans/${iface_name}`);
}
