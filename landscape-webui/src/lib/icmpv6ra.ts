export class IPV6RAServiceConfig {
  iface_name: string;
  enable: boolean;
  config: IPV6RAConfig;

  constructor(obj: {
    iface_name: string;
    enable?: boolean;
    config?: IPV6RAConfig;
  }) {
    this.iface_name = obj?.iface_name ?? "";
    this.enable = obj?.enable ?? true;
    this.config = new IPV6RAConfig(obj?.config ?? {});
  }
}

export class IPV6RAConfig {
  subnet_prefix: number;
  subnet_index: number;
  depend_iface: string;
  ra_preferred_lifetime: number;
  ra_valid_lifetime: number;
  constructor(obj?: {
    subnet_prefix?: number;
    subnet_index?: number;
    depend_iface?: string;
    ra_preferred_lifetime?: number;
    ra_valid_lifetime?: number;
  }) {
    this.subnet_prefix = obj?.subnet_prefix ?? 64;
    this.subnet_index = obj?.subnet_index ?? 0;
    this.depend_iface = obj?.depend_iface ?? "";
    this.ra_preferred_lifetime = obj?.ra_preferred_lifetime ?? 300;
    this.ra_valid_lifetime = obj?.ra_valid_lifetime ?? 600;
  }
}
