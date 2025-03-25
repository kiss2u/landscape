import { Netmask } from "netmask";

export class DHCPv4ServiceConfig {
  iface_name: string;
  enable: boolean;
  config: DHCPv4ServerConfig;

  constructor(obj?: {
    iface_name: string;
    enable?: boolean;
    config?: DHCPv4ServerConfig;
  }) {
    this.iface_name = obj?.iface_name ?? "";
    this.enable = obj?.enable ?? true;
    this.config = new DHCPv4ServerConfig(obj?.config);
  }
}

export class DHCPv4ServerConfig {
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
  return [sec_ip, block.broadcast];
}
