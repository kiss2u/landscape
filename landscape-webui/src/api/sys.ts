import api from "../api";

export async function get_sysinfo(): Promise<any> {
  let data = await api.api.get("sysinfo/sys");
  // console.log(data.data);
  return data.data;
}
export async function get_cpu(): Promise<any> {
  let data = await api.api.get("sysinfo/cpu");
  // console.log(data.data);
  return data.data;
}

export async function get_mem(): Promise<any> {
  let data = await api.api.get("sysinfo/mem");
  // console.log(data.data);
  return data.data;
}
