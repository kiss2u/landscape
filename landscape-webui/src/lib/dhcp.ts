import { Netmask } from "netmask";
import { IfaceIpMode } from "./service_ipconfig";

export class DhcpServerConfig {
  t: IfaceIpMode.DHCPServer;
  options: any[];
  server_ip_addr: string;
  network_mask: number;
  ip_range_start: string;
  ip_range_end: string | undefined;

  constructor(obj?: {
    options?: any[];
    server_ip_addr?: string;
    network_mask?: number;
    ip_range_start?: string;
    ip_range_end?: string;
  }) {
    this.t = IfaceIpMode.DHCPServer;
    this.options = obj?.options ?? [];
    this.server_ip_addr = obj?.server_ip_addr ?? "192.168.5.1";
    this.network_mask = obj?.network_mask ?? 24;
    const [start, end] = get_dhcp_range(
      `${this.server_ip_addr}/${this.network_mask}`
    );
    // console.log(end);
    this.ip_range_start = obj?.ip_range_start ?? start;
    this.ip_range_end = obj?.ip_range_end ?? end;
  }
}

export function get_dhcp_range(cidr: string): [string, string] {
  let block = new Netmask(cidr);
  let sec_ip = "";
  block.forEach((ip, _, index) => {
    if (index == 1) {
      sec_ip = ip;
    }
  });
  return [sec_ip, block.last];
}
