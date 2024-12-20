import { IfaceIpMode } from "./service_ipconfig";

export class DhcpServerConfig {
  t: IfaceIpMode.DHCPServer;
  server_ip: [number, number, number, number];
  network_mask: number;
  options: any[];
  host_range: { start: number; end: number };

  constructor(obj?: {
    server_ip?: [number, number, number, number];
    network_mask?: number;
    options?: any[];
    host_range?: { start: number; end: number };
  }) {
    this.t = IfaceIpMode.DHCPServer;
    this.server_ip = obj?.server_ip ?? [192, 168, 5, 1];
    this.network_mask = obj?.network_mask ?? 24;
    this.options = obj?.options ?? [];
    this.host_range = obj?.host_range ?? { start: 1, end: 254 };
  }
}
