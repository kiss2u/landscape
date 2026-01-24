import { LandscapeStatus, LandscapeSystemInfo } from "@/lib/sys";
import axiosService from ".";

export async function get_sysinfo(): Promise<LandscapeSystemInfo> {
  let data = await axiosService.get("sysinfo/sys");
  return data.data;
}

export async function interval_fetch_info(): Promise<LandscapeStatus> {
  let data = await axiosService.get("sysinfo/interval_fetch_info");
  return data.data;
}

export async function get_cpu_count(): Promise<number> {
  let data = await axiosService.get("sysinfo/cpu_count");
  return data.data;
}
