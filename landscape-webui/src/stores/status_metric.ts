import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { get_firewall_metric_status, get_metric_status } from "@/api/metric";
import { ServiceStatus, ServiceStatusType } from "@/lib/services";
import { FrontEndFirewallMetricServiceData } from "@/rust_bindings/common/metric";

export const useMetricStore = defineStore("dns_metric", () => {
  const metric_status = ref<ServiceStatus>(new ServiceStatus());
  const firewall_info = ref<FrontEndFirewallMetricServiceData>();

  const is_down = computed(() => {
    return metric_status.value.t == ServiceStatusType.Stop;
  });

  async function UPDATE_INFO() {
    metric_status.value = await get_metric_status();
    firewall_info.value = await get_firewall_metric_status();
  }

  return {
    is_down,
    metric_status,
    firewall_info,
    UPDATE_INFO,
  };
});
