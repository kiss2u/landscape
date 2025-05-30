const ADAY = 60 * 60 * 24;

export class PPPDServiceConfig {
  attach_iface_name: string;
  iface_name: string;
  enable: boolean;
  pppd_config: PPPDConfig;

  constructor(obj: {
    attach_iface_name: string;
    iface_name?: string;
    enable?: boolean;
    pppd_config?: PPPDConfig;
  }) {
    let date_str = (new Date().getTime() % ADAY).toString(36);
    this.attach_iface_name = obj.attach_iface_name;
    this.iface_name =
      obj?.iface_name ??
      `ppp-${obj.attach_iface_name}-${date_str}`.substring(0, 15);
    this.enable = obj?.enable ?? true;
    this.pppd_config = new PPPDConfig(obj?.pppd_config);
  }
}

export class PPPDConfig {
  default_route: boolean;
  peer_id: string;
  password: string;
  //   attach_iface_name: string;
  //   ppp_iface_name: string;

  constructor(obj?: {
    default_route?: boolean;
    peer_id?: string;
    password?: string;
    // attach_iface_name?: string;
    // ppp_iface_name?: string;
  }) {
    this.default_route = obj?.default_route ?? true;
    this.peer_id = obj?.peer_id ?? "";
    this.password = obj?.password ?? "";
    // this.attach_iface_name = obj?.attach_iface_name ?? "";
    // this.ppp_iface_name = obj?.ppp_iface_name ?? "";
  }
}
