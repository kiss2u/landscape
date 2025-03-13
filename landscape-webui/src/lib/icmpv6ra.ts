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
  ra_flag: IPV6RAConfigFlag;

  constructor(obj?: {
    subnet_prefix?: number;
    subnet_index?: number;
    depend_iface?: string;
    ra_preferred_lifetime?: number;
    ra_valid_lifetime?: number;
    ra_flag?: IPV6RAConfigFlag;
  }) {
    this.subnet_prefix = obj?.subnet_prefix ?? 64;
    this.subnet_index = obj?.subnet_index ?? 0;
    this.depend_iface = obj?.depend_iface ?? "";
    this.ra_preferred_lifetime = obj?.ra_preferred_lifetime ?? 300;
    this.ra_valid_lifetime = obj?.ra_valid_lifetime ?? 600;
    this.ra_flag = obj?.ra_flag
      ? new IPV6RAConfigFlag(obj?.ra_flag)
      : new IPV6RAConfigFlag(); // 1100 0000
  }
}

export class IPV6RAConfigFlag {
  managed_address_config: boolean; // 0b1000_0000
  other_config: boolean; // 0b0100_0000
  home_agent: boolean; // 0b0010_0000
  prf: number; // 0b0001_1000 (Default Router Preference)
  nd_proxy: boolean; // 0b0000_0100
  reserved: number; // 0b0000_0011

  constructor(obj?: {
    managed_address_config?: boolean; // 0b1000_0000
    other_config?: boolean; // 0b0100_0000
    home_agent?: boolean; // 0b0010_0000
    prf?: number; // 0b0001_1000 (Default Router Preference)
    nd_proxy?: boolean; // 0b0000_0100
  }) {
    this.managed_address_config = obj?.managed_address_config ?? true;
    this.other_config = obj?.other_config ?? true;
    this.home_agent = obj?.home_agent ?? false;
    this.prf = obj?.prf ?? 0;
    this.nd_proxy = obj?.nd_proxy ?? false;
    this.reserved = 0;
  }
}
