import axiosService from "@/api";
import { ServiceStatus } from "@/lib/services";
import {
  ConnectKey,
  ConnectMetric,
} from "@/rust_bindings/common/metric/connect";

export async function get_metric_status(): Promise<ServiceStatus> {
  let data = await axiosService.get("metric/status");
  // console.log(data.data);
  return data.data;
}

export async function get_connects_info(): Promise<ConnectKey[]> {
  let data = await axiosService.get("metric/connects");
  return data.data;
}

export async function get_connect_metric_info(
  key: ConnectKey
): Promise<ConnectMetric[]> {
  let data = await axiosService.post("metric/connects/chart", {
    ...key,
  });
  // console.log(data.data);
  return data.data;
}
