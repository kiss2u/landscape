import { MarkType, PacketMark } from "./dns";

export class MarkServiceConfig {
  iface_name: string;
  enable: boolean;

  constructor(obj: { iface_name: string; enable?: boolean }) {
    this.iface_name = obj?.iface_name ?? "";
    this.enable = obj?.enable ?? true;
  }
}

export class IpConfig {
  ip: string;
  prefix: number;

  constructor(obj?: { ip?: string; prefix?: number }) {
    this.ip = obj?.ip ?? "0.0.0.0";
    this.prefix = obj?.prefix ?? 32;
  }
}

export class LanIPRuleConfig {
  index: number;
  enable: boolean;
  mark: PacketMark;
  source: IpConfig[];
  remark: string;

  constructor(obj?: {
    index?: number;
    enable?: boolean;
    mark?: PacketMark;
    source?: IpConfig[];
    remark?: string;
  }) {
    this.index = obj?.index ?? -1;
    this.enable = obj?.enable ?? true;
    this.mark = obj?.mark ? { ...obj.mark } : { t: MarkType.NoMark };
    this.source = obj?.source ? obj?.source.map((e) => new IpConfig(e)) : [];
    this.remark = obj?.remark ?? "";
  }
}

export class WanIPRuleConfig {
  index: number;
  enable: boolean;
  mark: PacketMark;
  source: WanIPRuleSource[];
  remark: string;

  constructor(obj?: {
    index?: number;
    enable?: boolean;
    mark?: PacketMark;
    source?: WanIPRuleSource[];
    remark?: string;
  }) {
    this.index = obj?.index ?? -1;
    this.enable = obj?.enable ?? true;
    this.mark = obj?.mark ? { ...obj.mark } : { t: MarkType.NoMark };
    this.source = obj?.source ? obj?.source.map(new_wan_rules) : [];
    this.remark = obj?.remark ?? "";
  }
}

export type WanIPRuleSource =
  | { t: "geokey"; key: string }
  | { t: "config"; ip: string; prefix: number };

export function new_wan_rules(e: any): WanIPRuleSource {
  if (e.t == "config") {
    return { t: "config", ip: e.ip, prefix: e.prefix };
  } else {
    return { t: "geokey", key: e.key };
  }
}
