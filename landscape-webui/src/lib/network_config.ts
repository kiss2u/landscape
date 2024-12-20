export type IpConfigMode =
  | { t: "nothing" }
  | { t: "static"; ipv4: number[]; ipv4_mask: number; ipv6: number[] }
  | { t: "pppoe"; username: string; password: string; mtu: number }
  | { t: "dhcp" };

export class NetworkConfig {
  name: string;
  master: string | undefined;
  mac: string | undefined;
  perm_mac: string | undefined;
  ip_config_mode: IpConfigMode;

  constructor(obj: any) {
    this.name = obj.name;
    this.master = obj.master;
    this.mac = obj.mac;
    this.perm_mac = obj.perm_mac;
    this.ip_config_mode = obj.ip_config_mode ?? { t: "nothing" };
  }
}

// export class IfaceAttribute {
//     //
//     t: string;
// }
