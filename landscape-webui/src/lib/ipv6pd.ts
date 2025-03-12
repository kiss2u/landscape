export class IPV6PDServiceConfig {
  iface_name: string;
  enable: boolean;
  config: IPV6PDConfig;

  constructor(obj: {
    iface_name: string;
    enable?: boolean;
    config?: IPV6PDConfig;
  }) {
    this.iface_name = obj?.iface_name ?? "";
    this.enable = obj?.enable ?? true;
    this.config = new IPV6PDConfig(obj?.config ?? {});
  }
}

export class IPV6PDConfig {
  mac: string;

  constructor(obj?: { mac?: string }) {
    this.mac = obj?.mac ?? "";
  }
}
