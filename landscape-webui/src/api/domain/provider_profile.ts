import {
  createProviderProfile,
  deleteProviderProfile,
  getProviderProfile,
  listProviderProfiles,
  updateProviderProfile,
} from "@landscape-router/types/api/dns-provider-profiles/dns-provider-profiles";
import type { DnsProviderProfile } from "@landscape-router/types/api/schemas";

export async function get_dns_provider_profiles(): Promise<
  DnsProviderProfile[]
> {
  return listProviderProfiles();
}

export async function get_dns_provider_profile(id: string) {
  return getProviderProfile(id);
}

export async function push_dns_provider_profile(payload: DnsProviderProfile) {
  if (payload.id) {
    return updateProviderProfile(payload.id, payload);
  }
  return createProviderProfile(payload);
}

export async function delete_dns_provider_profile(id: string): Promise<void> {
  await deleteProviderProfile(id);
}
