import type {
  GetDnsConfigResponse,
  GetDnsConfigResponse as GetDnsConfigFastResponse,
  GetGatewayConfigResponse,
  GetMetricConfigResponse,
  LandscapeGatewayConfig,
  GetUIConfigResponse,
  LandscapeDnsConfig,
  LandscapeMetricConfig,
  LandscapeUIConfig,
  UpdateGatewayConfigRequest,
  UpdateMetricConfigRequest,
  UpdateUIConfigRequest,
} from "@landscape-router/types/api/schemas";
import {
  exportInitConfig,
  getUiConfigFast,
  getUiConfig,
  updateUiConfig,
  getMetricConfigFast,
  getMetricConfig,
  updateMetricConfig,
  getDnsConfigFast,
  getDnsConfig,
  updateDnsConfig,
  getGatewayConfigFast,
  getGatewayConfig,
  updateGatewayConfig,
} from "@landscape-router/types/api/system-config/system-config";

/** Local type -- backend accepts serde_json::Value, so no ORVAL-generated request type exists. */
interface UpdateDnsConfigRequest {
  new_dns: LandscapeDnsConfig;
  expected_hash: string;
}

export async function get_init_config(): Promise<void> {
  try {
    const jsonStr = await exportInitConfig();

    const filename = "landscape_init.toml";

    const blob = new Blob([jsonStr], { type: "application/octet-stream" });
    const url = window.URL.createObjectURL(blob);

    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();

    window.URL.revokeObjectURL(url);
  } catch (error) {
    console.error("下载配置失败", error);
  }
}

export async function get_ui_config(): Promise<LandscapeUIConfig> {
  return await getUiConfigFast();
}

export async function get_ui_config_edit(): Promise<GetUIConfigResponse> {
  return await getUiConfig();
}

export async function update_ui_config(
  payload: UpdateUIConfigRequest,
): Promise<void> {
  await updateUiConfig(payload);
}

export async function get_metric_config(): Promise<LandscapeMetricConfig> {
  return await getMetricConfigFast();
}

export async function get_metric_config_edit(): Promise<GetMetricConfigResponse> {
  return await getMetricConfig();
}

export async function update_metric_config(
  payload: UpdateMetricConfigRequest,
): Promise<void> {
  await updateMetricConfig(payload);
}

export async function get_dns_config(): Promise<GetDnsConfigFastResponse> {
  return await getDnsConfigFast();
}

export async function get_dns_config_edit(): Promise<GetDnsConfigResponse> {
  return await getDnsConfig();
}

export async function update_dns_config(
  payload: UpdateDnsConfigRequest,
): Promise<void> {
  await updateDnsConfig(payload);
}

export async function get_gateway_config(): Promise<LandscapeGatewayConfig> {
  return await getGatewayConfigFast();
}

export async function get_gateway_config_edit(): Promise<GetGatewayConfigResponse> {
  return await getGatewayConfig();
}

export async function update_gateway_config(
  payload: UpdateGatewayConfigRequest,
): Promise<void> {
  await updateGatewayConfig(payload);
}
