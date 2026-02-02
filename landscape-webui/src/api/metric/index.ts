import axiosService from "@/api";
import { ServiceStatus } from "@/lib/services";
import {
  ConnectKey,
  ConnectMetric,
  ConnectRealtimeStatus,
  ConnectHistoryStatus,
  ConnectGlobalStats,
  ConnectHistoryQueryParams,
} from "landscape-types/common/metric/connect";

export * from './dns';

export async function get_connect_global_stats(): Promise<ConnectGlobalStats> {
  let data = await axiosService.get("metric/connects/global_stats");
  return data.data;
}

export async function get_metric_status(): Promise<ServiceStatus> {
  let data = await axiosService.get("metric/status");
  return data.data;
}

export async function get_connects_info(): Promise<ConnectRealtimeStatus[]> {
  let data = await axiosService.get("metric/connects");
  return data.data;
}

export async function get_connect_history(
  params?: ConnectHistoryQueryParams,
): Promise<ConnectHistoryStatus[]> {
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
