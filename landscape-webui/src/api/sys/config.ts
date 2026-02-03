import axiosService from "@/api";
import type {
  GetMetricConfigResponse,
  GetUIConfigResponse,
  LandscapeMetricConfig,
  LandscapeUIConfig,
  UpdateMetricConfigRequest,
  UpdateUIConfigRequest,
} from "landscape-types/common/config";

export async function get_init_config(): Promise<void> {
  try {
    const response = await axiosService.get(`sys_service/config/export`);
    const jsonStr = response.data;

    let filename = "landscape_init.toml";

    const blob = new Blob([jsonStr], { type: "application/octet-stream" });
    const url = window.URL.createObjectURL(blob);

    // 创建 a 标签模拟点击
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();

    // 释放 URL
    window.URL.revokeObjectURL(url);
  } catch (error) {
    console.error("下载配置失败", error);
  }
}

export async function get_ui_config(): Promise<LandscapeUIConfig> {
  const response = await axiosService.get(`sys_service/config/ui`);
  return response.data;
}

export async function get_ui_config_edit(): Promise<GetUIConfigResponse> {
  const response = await axiosService.get(`sys_service/config/edit/ui`);
  return response.data;
}

export async function update_ui_config(
  payload: UpdateUIConfigRequest,
): Promise<void> {
  await axiosService.post(`sys_service/config/edit/ui`, payload);
}

export async function get_metric_config(): Promise<LandscapeMetricConfig> {
  const response = await axiosService.get(`sys_service/config/metric`);
  return response.data;
}

export async function get_metric_config_edit(): Promise<GetMetricConfigResponse> {
  const response = await axiosService.get(`sys_service/config/edit/metric`);
  return response.data;
}

export async function update_metric_config(
  payload: UpdateMetricConfigRequest,
): Promise<void> {
  await axiosService.post(`sys_service/config/edit/metric`, payload);
}
