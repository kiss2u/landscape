import api from ".";
import { DnsRule } from "@/lib/dns";
import { ServiceStatus } from "@/lib/services";

export async function get_dns_status(): Promise<ServiceStatus> {
  let data = await api.api.get("dns");
  // console.log(data.data);
  return new ServiceStatus(data.data.status);
}

export async function start_dns_service(
  udp_port: number
): Promise<ServiceStatus> {
  let data = await api.api.post("dns", {
    udp_port,
  });
  // console.log(data.data);
  return new ServiceStatus(data.data.status);
}

export async function stop_dns_service(): Promise<ServiceStatus> {
  let data = await api.api.delete("dns");
  // console.log(data.data);
  return new ServiceStatus(data.data.status);
}

export async function get_dns_rule(): Promise<DnsRule[]> {
  let data = await api.api.get("dns/rules");
  return data.data.map((d: any) => new DnsRule(d));
}

export async function push_dns_rule(rule: DnsRule): Promise<void> {
  let data = await api.api.post("dns/rules", rule);
}

export async function delete_dns_rule(index: number): Promise<void> {
  let data = await api.api.delete(`dns/rules/${index}`);
}
