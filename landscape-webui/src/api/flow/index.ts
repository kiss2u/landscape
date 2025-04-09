import api from "@/api";
import { FlowConfig } from "@/rust_bindings/flow";

export async function get_flow_rules(): Promise<FlowConfig[]> {
  let data = await api.api.get("flow");
  return data.data;
}
export async function push_flow_rules(config: FlowConfig): Promise<void> {
  await api.api.post("flow", config);
}
