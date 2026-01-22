import axiosService from "@/api";
import { ServiceStatus } from "@/lib/services";
import { RouteWanServiceConfig } from "landscape-types/common/route";

export async function get_all_route_wan_status(): Promise<
  Map<string, ServiceStatus>
> {
  let data = await axiosService.get(`services/route_wans/status`);
  let map = new Map<string, ServiceStatus>();
  for (const [key, value] of Object.entries(data.data)) {
    map.set(key, new ServiceStatus(value as any));
  }
  return map;
}

export async function get_route_wan_config(
  id: string
): Promise<RouteWanServiceConfig> {
  let result = await axiosService.get(`services/route_wans/${id}`);
  return result.data;
}

export async function update_route_wans_config(
  config: RouteWanServiceConfig
): Promise<void> {
  await axiosService.post(`services/route_wans`, config);
}

export async function del_route_wans(iface_name: string): Promise<void> {
  await axiosService.delete(`services/route_wans/${iface_name}`);
}
