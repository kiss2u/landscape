import { get_all_route_lan_status } from "@/api/route/lan";
import { ServiceStatus } from "@/lib/services";
import { defineStore } from "pinia";
import { computed, ComputedRef, ref } from "vue";

export const useRouteLanConfigStore = defineStore("status_route_lan", () => {
  const status = ref<Map<string, ServiceStatus>>(
    new Map<string, ServiceStatus>(),
  );

  async function UPDATE_INFO() {
    status.value = await get_all_route_lan_status();
  }

  function GET_STATUS_BY_IFACE_NAME(
    name: string,
  ): ComputedRef<ServiceStatus | undefined> {
    return computed(() => status.value.get(name));
  }

  return {
    UPDATE_INFO,
    GET_STATUS_BY_IFACE_NAME,
  };
});

// const routeLanConfigStore = useRouteLanConfigStore();
