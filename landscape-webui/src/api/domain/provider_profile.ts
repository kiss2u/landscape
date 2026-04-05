import {
  createDnsProviderProfile,
  deleteDnsProviderProfile,
  getDnsProviderProfile,
  listDnsProviderProfiles,
  updateDnsProviderProfile,
} from "@landscape-router/types/api/dns-provider-profiles/dns-provider-profiles";
import type { DnsProviderProfile } from "@landscape-router/types/api/schemas";

export async function get_dns_provider_profiles(): Promise<
  DnsProviderProfile[]
> {
  return listDnsProviderProfiles();
}

export async function get_dns_provider_profile(id: string) {
  return getDnsProviderProfile(id);
}

export async function push_dns_provider_profile(payload: DnsProviderProfile) {
  if (payload.id) {
    return updateDnsProviderProfile(payload.id, payload);
  }
  return createDnsProviderProfile(payload);
}

export async function delete_dns_provider_profile(id: string): Promise<void> {
  await deleteDnsProviderProfile(id);
}
