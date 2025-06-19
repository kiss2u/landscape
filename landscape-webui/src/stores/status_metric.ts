import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { get_connects_info, get_metric_status } from "@/api/metric";
import { ServiceStatus, ServiceStatusType } from "@/lib/services";
import { ConnectKey } from "@/rust_bindings/common/metric/connect";

export const useMetricStore = defineStore("dns_metric", () => {
  const enable = ref(false);
  const metric_status = ref<ServiceStatus>(new ServiceStatus());
  const firewall_info = ref<ConnectKey[]>();

  const is_down = computed(() => {
    return metric_status.value.t == ServiceStatusType.Stop;
  });

  async function UPDATE_INFO() {
    if (enable.value) {
      metric_status.value = await get_metric_status();
      firewall_info.value = await get_connects_info();
    }
  }

  async function SET_ENABLE(value: boolean) {
    enable.value = value;
  }

  return {
    SET_ENABLE,
    is_down,
    metric_status,
    firewall_info,
    UPDATE_INFO,
  };
});
