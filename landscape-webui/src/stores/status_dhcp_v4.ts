import { get_all_dhcp_v4_status } from "@/api/service_dhcp_v4";
import { DHCPv4ServiceStatus } from "@/lib/dhcp_v4";
import { defineStore } from "pinia";
import { computed, ComputedRef, ref } from "vue";

export const useDHCPv4ConfigStore = defineStore("status_dhcp_v4", () => {
  const status = ref<Map<string, DHCPv4ServiceStatus>>(
    new Map<string, DHCPv4ServiceStatus>()
  );

  async function UPDATE_INFO() {
    status.value = await get_all_dhcp_v4_status();
  }

  function GET_STATUS_BY_IFACE_NAME(
    name: string
  ): ComputedRef<DHCPv4ServiceStatus | undefined> {
    return computed(() => status.value.get(name));
  }

  return {
    UPDATE_INFO,
    GET_STATUS_BY_IFACE_NAME,
  };
});

// const dhcpv4ConfigStore = useDHCPv4ConfigStore();
