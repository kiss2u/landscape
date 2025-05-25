import api from "@/api";
import {
  GeoDomainConfig,
  GeoSiteConfig,
} from "@/rust_bindings/common/geo_site";

export async function get_geo_site_configs(): Promise<GeoSiteConfig[]> {
  let data = await api.api.get(`config/geo_sites`);
  return data.data;
}

export async function get_geo_site_config(id: string): Promise<GeoSiteConfig> {
  let data = await api.api.get(`config/geo_sites/${id}`);
  return data.data;
}

export async function push_geo_site_config(
  config: GeoSiteConfig
): Promise<void> {
  let data = await api.api.post(`config/geo_sites`, config);
}

export async function delete_geo_site_config(id: string): Promise<void> {
  let data = await api.api.delete(`config/geo_sites/${id}`);
}

export async function get_geo_cache_key(): Promise<GeoDomainConfig[]> {
  let data = await api.api.get(`config/geo_sites/cache`);
  return data.data;
}

export async function refresh_geo_cache_key(): Promise<void> {
  let data = await api.api.post(`config/geo_sites/cache`);
}
