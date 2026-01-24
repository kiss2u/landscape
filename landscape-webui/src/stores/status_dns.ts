import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { get_dns_status } from "@/api/dns_service";
import { ServiceStatus, ServiceStatusType } from "@/lib/services";

export const useDnsStore = defineStore("dns_status", () => {
  const dns_status = ref<ServiceStatus>({ t: ServiceStatusType.Stop });

  const is_down = computed(() => {
    return dns_status.value.t == ServiceStatusType.Stop;
  });

  async function UPDATE_INFO() {
    dns_status.value = await get_dns_status();
  }

  return {
    is_down,
    dns_status,
    UPDATE_INFO,
  };
});
