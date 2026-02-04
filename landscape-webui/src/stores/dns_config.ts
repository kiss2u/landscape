import { defineStore } from "pinia";
import { ref } from "vue";
import type { LandscapeDnsConfig } from "landscape-types/common/config";
import { get_dns_config_edit, update_dns_config } from "@/api/sys/config";

export const useDnsConfigStore = defineStore("dns_config", () => {
  const cacheCapacity = ref<number | undefined>(undefined);
  const cacheTtl = ref<number | undefined>(undefined);
  const expectedHash = ref<string>("");

  async function loadDnsConfig() {
    const { dns, hash } = await get_dns_config_edit();
    cacheCapacity.value = dns.cache_capacity;
    cacheTtl.value = dns.cache_ttl;
    expectedHash.value = hash;
  }

  async function saveDnsConfig() {
    const new_dns: LandscapeDnsConfig = {
      cache_capacity: cacheCapacity.value || undefined,
      cache_ttl: cacheTtl.value || undefined,
    };
    await update_dns_config({
      new_dns,
      expected_hash: expectedHash.value,
    });

    // Refresh hash after save
    const { hash } = await get_dns_config_edit();
    expectedHash.value = hash;
  }

  return {
    cacheCapacity,
    cacheTtl,
    expectedHash,
    loadDnsConfig,
    saveDnsConfig,
  };
});
