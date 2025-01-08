import { Range } from "@/lib/common";
const DEFAULT_RANGE_START = 32768;
const DEFAULT_RANGE_END = 65535;

export class NatServiceConfig {
  iface_name: string;
  enable: boolean;
  nat_config: NatConfig;

  constructor(obj: {
    iface_name: string;
    enable?: boolean;
    nat_config?: NatConfig;
  }) {
    this.iface_name = obj?.iface_name ?? "";
    this.enable = obj?.enable ?? true;
    this.nat_config = new NatConfig(obj?.nat_config ?? {});
  }
}

export class NatConfig {
  tcp_range: Range;
  udp_range: Range;
  icmp_in_range: Range;

  constructor(obj?: {
    tcp_range?: Range;
    udp_range?: Range;
    icmp_in_range?: Range;
  }) {
    this.tcp_range =
      obj?.tcp_range ?? new Range(DEFAULT_RANGE_START, DEFAULT_RANGE_END);
    this.udp_range =
      obj?.udp_range ?? new Range(DEFAULT_RANGE_START, DEFAULT_RANGE_END);
    this.icmp_in_range =
      obj?.icmp_in_range ?? new Range(DEFAULT_RANGE_START, DEFAULT_RANGE_END);
  }
}
