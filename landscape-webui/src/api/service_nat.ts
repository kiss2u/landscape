import { NatServiceConfig } from "@/lib/nat";
import api from ".";
import { ServiceStatus } from "@/lib/services";

export async function get_all_nat_status(): Promise<
  Map<string, ServiceStatus>
> {
  let data = await api.api.get(`services/nats/status`);
  let map = new Map<string, ServiceStatus>();
  for (const [key, value] of Object.entries(data.data)) {
    map.set(key, new ServiceStatus(value as any));
  }
  return map;
}

export async function get_iface_nat_config(
  iface_name: string
): Promise<NatServiceConfig> {
  let data = await api.api.get(`services/nats/${iface_name}`);
  console.log(data.data);
  return data.data;
}

export async function update_iface_nat_config(
  iface_name: string,
  nat_config: NatServiceConfig
): Promise<void> {
  let data = await api.api.post(`services/nats/${iface_name}`, {
    ...nat_config,
  });
  console.log(data.data);
  return data.data;
}

export async function stop_and_del_iface_nat(name: string): Promise<void> {
  return api.api.delete(`services/nats/${name}`);
}
