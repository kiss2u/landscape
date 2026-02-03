import { defineStore } from "pinia";
import { ref, computed } from "vue";
import {
  get_connects_info,
  get_metric_status,
  get_connect_metric_info,
  get_src_ip_stats,
  get_dst_ip_stats,
} from "@/api/metric";
import { ServiceStatus, ServiceStatusType } from "@/lib/services";
import {
  ConnectKey,
  ConnectRealtimeStatus,
  IpRealtimeStat,
} from "landscape-types/common/metric/connect";

export const useMetricStore = defineStore("dns_metric", () => {
  const enable = ref(false);
  const metric_status = ref<ServiceStatus>({ t: ServiceStatusType.Stop });
  const firewall_info = ref<ConnectRealtimeStatus[]>();
  const src_ip_stats = ref<IpRealtimeStat[]>([]);
  const dst_ip_stats = ref<IpRealtimeStat[]>([]);

  const is_down = computed(() => {
    return metric_status.value.t == ServiceStatusType.Stop;
  });

  async function UPDATE_INFO() {
    if (enable.value) {
      metric_status.value = await get_metric_status();
      const [firewall, src_ips, dst_ips] = await Promise.all([
        get_connects_info(),
        get_src_ip_stats(),
        get_dst_ip_stats(),
      ]);
      firewall_info.value = firewall;
      src_ip_stats.value = src_ips;
      dst_ip_stats.value = dst_ips;
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
    src_ip_stats,
    dst_ip_stats,
    UPDATE_INFO,
  };
});
