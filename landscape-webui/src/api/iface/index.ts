import api from "@/api";
import { IfacesInfo } from "@/rust_bindings/iface";

export async function ifaces(): Promise<IfacesInfo> {
  let data = await api.api.get("iface");
  // console.log(data.data);
  return data.data;
}

export async function manage_iface(dev_name: String): Promise<IfacesInfo> {
  let data = await api.api.post(`iface/manage/${dev_name}`);
  // console.log(data.data);
  return data.data;
}
