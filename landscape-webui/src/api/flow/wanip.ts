import api from "@/api";
import { WanIpRuleConfigClass } from "@/lib/mark";

import { WanIpRuleConfig } from "landscape-types/common/flow";

// export async function get_wan_ip_rules(
//   flow_id: number
// ): Promise<WanIpRuleConfig[]> {
//   let data = await api.api.get(`flow/${flow_id}/wans`);
//   //   console.log(data.data);
//   return data.data.map((e: any) => new WanIpRuleConfigClass(e));
// }

// export async function get_wan_ip_rule(
//   flow_id: number,
//   rule_id: string
// ): Promise<WanIpRuleConfig> {
//   let data = await api.api.get(`flow/${flow_id}/wans/${rule_id}`);
//   //   console.log(data.data);
//   return new WanIpRuleConfigClass(data.data);
// }

// export async function post_wan_ip_rules(
//   flow_id: number,
//   data: WanIpRuleConfig
// ): Promise<void> {
//   let result = await api.api.post(`flow/${flow_id}/wans`, data);
//   //   console.log(data.data);
// }

// export async function update_wan_ip_rules(
//   flow_id: number,
//   rule_id: string,
//   data: WanIpRuleConfig
// ): Promise<void> {
//   let result = await api.api.put(`flow/${flow_id}/wans/${rule_id}`, data);
//   //   console.log(data.data);
// }

// export async function del_wan_ip_rules(index: number): Promise<void> {
//   let result = await api.api.delete(`flow/wans/${index}`);
//   //   console.log(data.data);
// }
