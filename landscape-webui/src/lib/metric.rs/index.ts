export class ConnectFilter {
  src_ip: string | null;
  dst_ip: string | null;
  port_start: number | null;
  port_end: number | null;
  l3_proto: number | null;
  l4_proto: number | null;
  flow_id: number | null;

  constructor(obj: Partial<ConnectFilter> = {}) {
    this.src_ip = obj.src_ip ?? null;
    this.dst_ip = obj.dst_ip ?? null;
    this.port_start = obj.port_start ?? null;
    this.port_end = obj.port_end ?? null;
    this.l3_proto = obj.l3_proto ?? null;
    this.l4_proto = obj.l4_proto ?? null;
    this.flow_id = obj.flow_id ?? null;
  }
}
