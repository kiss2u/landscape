import api from "@/api";
import { WanIpRuleConfigClass } from "@/lib/mark";
import { WanIpRuleConfig } from "@/rust_bindings/common/flow";

export async function get_flow_dst_ip_rules(
  flow_id: number
): Promise<WanIpRuleConfig[]> {
  let data = await api.api.get(`config/dst_ip_rules/flow/${flow_id}`);
  return data.data.map((d: any) => new WanIpRuleConfigClass(d));
}

export async function get_dst_ip_rules_rule(
  id: string
): Promise<WanIpRuleConfig> {
  let data = await api.api.get(`config/dst_ip_rules/${id}`);
  return new WanIpRuleConfigClass(data.data);
}

export async function push_dst_ip_rules_rule(
  rule: WanIpRuleConfig
): Promise<void> {
  let data = await api.api.post(`config/dst_ip_rules`, rule);
}

export async function update_dst_ip_rules_rule(
  id: string,
  rule: WanIpRuleConfig
): Promise<void> {
  let data = await api.api.post(`config/dst_ip_rules/${id}`, rule);
}

export async function delete_dst_ip_rules_rule(id: string): Promise<void> {
  let data = await api.api.delete(`config/dst_ip_rules/${id}`);
}
