import axiosService from "@/api";
import { StaticNatMappingConfig } from "landscape-types/common/nat";

export async function get_static_nat_mappings(): Promise<
  StaticNatMappingConfig[]
> {
  let data = await axiosService.get(`config/static_nat_mappings`);
  return data.data;
}

export async function get_static_nat_mapping(
  id: string
): Promise<StaticNatMappingConfig> {
  let data = await axiosService.get(`config/static_nat_mappings/${id}`);
  return data.data;
}

export async function push_static_nat_mapping(
  rule: StaticNatMappingConfig
): Promise<void> {
  let data = await axiosService.post(`config/static_nat_mappings`, rule);
}

export async function delete_static_nat_mapping(id: string): Promise<void> {
  let data = await axiosService.delete(`config/static_nat_mappings/${id}`);
}

export async function push_many_static_nat_mapping(
  rule: StaticNatMappingConfig[]
): Promise<void> {
  let data = await axiosService.post(
    `config/static_nat_mappings/set_many`,
    rule
  );
}
