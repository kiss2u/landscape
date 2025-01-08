export class DnsRule {
  index: number;
  name: string;
  enable: boolean;
  mark: PacketMark;
  dns_resolve_ip: string;
  source: RuleSource[];

  constructor(obj?: {
    index?: number;
    name?: string;
    enable?: boolean;
    mark?: PacketMark;
    dns_resolve_ip?: string;
    source?: RuleSource[];
  }) {
    this.index = obj?.index ?? -1;
    this.name = obj?.name ?? "";
    this.enable = obj?.enable ?? true;
    this.mark = obj?.mark ? { ...obj.mark } : { t: MarkType.NoMark };
    this.dns_resolve_ip = obj?.dns_resolve_ip ?? "1.1.1.1";
    this.source = obj?.source ?? [];
  }
}

export enum DomainMatchType {
  Plain = "plain",
  Regex = "regex",
  Domain = "domain",
  Full = "full",
}

export type RuleSource =
  | { t: "geokey"; key: string }
  | { t: "config"; match_type: DomainMatchType; value: string };

export enum MarkType {
  NoMark = "nomark",
  /// 直连
  Direct = "direct",
  /// 丢弃数据包
  Drop = "drop",
  /// 转发到另一张网卡中
  Redirect = "redirect",
  /// 进行 IP 校验 ( 阻止进行打洞 )
  SymmetricNat = "symmetricnat",
  RedirectNetns = "redirectnetns",
}

export type PacketMark =
  | { t: MarkType.NoMark }
  | { t: MarkType.Direct }
  | { t: MarkType.Drop }
  | { t: MarkType.Redirect; index: number }
  | { t: MarkType.SymmetricNat }
  | { t: MarkType.RedirectNetns; index: number };
