export class ConnectFilter {
  src_ip: string | undefined;
  dst_ip: string | undefined;
  port_start: number | undefined;
  port_end: number | undefined;
  l3_proto: number | undefined;
  l4_proto: number | undefined;
  flow_id: number | undefined;

  constructor(obj: Partial<ConnectFilter> = {}) {
    this.src_ip = obj.src_ip;
    this.dst_ip = obj.dst_ip;
    this.port_start = obj.port_start;
    this.port_end = obj.port_end;
    this.l3_proto = obj.l3_proto;
    this.l4_proto = obj.l4_proto;
    this.flow_id = obj.flow_id;
  }
}
