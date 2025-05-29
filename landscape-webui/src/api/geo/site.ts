import api from "@/api";
import {
  GeoDomainConfig,
  GeoSiteConfig,
  GeoDomainConfigKey,
  QueryGeoDomain,
} from "@/rust_bindings/common/geo_site";

export async function get_geo_site_configs(
  name?: string
): Promise<GeoSiteConfig[]> {
  let data = await api.api.get(`config/geo_sites`, {
    params: {
      name,
    },
  });
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

export async function get_geo_cache_key(
  filter: QueryGeoDomain
): Promise<GeoDomainConfigKey[]> {
  let data = await api.api.get(`config/geo_sites/cache`, {
    params: { ...filter },
  });
  return data.data;
}

export async function refresh_geo_cache_key(): Promise<void> {
  let data = await api.api.post(`config/geo_sites/cache`);
}

export async function search_geo_site_cache(
  query: QueryGeoDomain
): Promise<GeoDomainConfigKey[]> {
  let data = await api.api.get(`config/geo_sites/cache/search`, {
    params: {
      ...query,
    },
  });
  return data.data;
}

export async function get_geo_site_cache_detail(
  key: GeoDomainConfigKey
): Promise<GeoDomainConfig> {
  let data = await api.api.get(`config/geo_sites/cache/detail`, {
    params: {
      ...key,
    },
  });
  return data.data;
}
