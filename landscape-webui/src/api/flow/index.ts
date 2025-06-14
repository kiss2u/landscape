import api from "@/api";
import { FlowConfig } from "@/rust_bindings/common/flow";

export async function get_flow_rules(): Promise<FlowConfig[]> {
  let data = await api.api.get("config/flow_rules");
  return data.data;
}

export async function get_flow_rule(id: string): Promise<FlowConfig> {
  let result = await api.api.get(`config/flow_rules/${id}`);
  return result.data;
}

export async function push_flow_rules(config: FlowConfig): Promise<void> {
  await api.api.post(`config/flow_rules`, config);
}

export async function del_flow_rules(id: string): Promise<void> {
  await api.api.delete(`config/flow_rules/${id}`);
}
