import axiosService from "@/api";
import { ServiceStatus } from "@/lib/services";
import {
  ConnectKey,
  ConnectMetric,
  ConnectRealtimeStatus,
  ConnectHistoryStatus,
  ConnectGlobalStats,
} from "landscape-types/common/metric/connect";

export * from './dns';

export async function get_connect_global_stats(): Promise<ConnectGlobalStats> {
  let data = await axiosService.get("metric/connects/global_stats");
  return data.data;
}

export async function get_metric_status(): Promise<ServiceStatus> {
  let data = await axiosService.get("metric/status");
  // console.log(data.data);
  return data.data;
}

export async function get_connects_info(): Promise<ConnectRealtimeStatus[]> {
  let data = await axiosService.get("metric/connects");
  return data.data;
}

export async function get_connect_history(params?: {
  start_time?: number;
  end_time?: number;
  limit?: number;
  src_ip?: string;
  dst_ip?: string;
  port_start?: number;
  port_end?: number;
  l3_proto?: number;
  l4_proto?: number;
  flow_id?: number;
  sort_key?: string;
  sort_order?: string;
}): Promise<ConnectHistoryStatus[]> {
  let data = await axiosService.get("metric/connects/history", {
    params,
  });
  return data.data;
}

export async function get_connect_metric_info(
  key: ConnectKey,
): Promise<ConnectMetric[]> {
  let data = await axiosService.post("metric/connects/chart", {
    ...key,
  });
  // console.log(data.data);
  return data.data;
}
