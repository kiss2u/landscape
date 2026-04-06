import {
  createProviderProfile,
  deleteProviderProfile,
  getProviderProfile,
  listProviderProfiles,
  updateProviderProfile,
  validateProviderProfile,
} from "@landscape-router/types/api/dns-provider-profiles/dns-provider-profiles";
import type {
  DnsProviderCredentialCheckRequest,
  DnsProviderCredentialCheckResult,
  DnsProviderProfile,
} from "@landscape-router/types/api/schemas";

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

export async function validate_dns_provider_profile_credentials(
  payload: DnsProviderCredentialCheckRequest,
): Promise<DnsProviderCredentialCheckResult> {
  return validateProviderProfile(payload);
}
