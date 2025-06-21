<script setup lang="ts">
import { ZoneType } from "@/lib/service_ipconfig";
import { H, NetworkEnterprise } from "@vicons/carbon";

import StatusBtn from "@/components/status_btn/StatusBtn.vue";
import { useDHCPv4ConfigStore } from "@/stores/status_dhcp_v4";
import { NCountdown } from "naive-ui";
import { computed, h, ref } from "vue";
import {
  DHCPv4OfferInfo,
  DHCPv4OfferInfoShow,
  conver_to_show,
} from "@/lib/dhcp_v4";
import { ServiceStatusType } from "@/lib/services";
import { IfaceZoneType } from "@/rust_bindings/common/iface";
import { get_dhcp_v4_assigned_ips_by_iface_name } from "@/api/service_dhcp_v4";

const dhcpv4ConfigStore = useDHCPv4ConfigStore();

const iface_info = defineProps<{
  iface_name: string;
  zone: IfaceZoneType;
}>();

const status = dhcpv4ConfigStore.GET_STATUS_BY_IFACE_NAME(
  iface_info.iface_name
);
const emit = defineEmits(["click"]);
const columns = [
  {
    title: "Mac 地址",
    key: "mac",
  },
  {
    title: "分配 IP",
    key: "ip",
  },
  {
    title: "分配租期时间 (s)",
    key: "time_left",
    // render(row: DHCPv4OfferInfoShow) {
    //   return h(NCountdown, {
    //     duration: row.time_left * 1000,
    //   });
    // },
  },
];

const disable_popover = computed(() => {
  return status.value?.status.t === ServiceStatusType.Running ? false : true;
});

const assigned_ips = ref<DHCPv4OfferInfo | null>(null);
async function read_assigned_ips() {
  assigned_ips.value = await get_dhcp_v4_assigned_ips_by_iface_name(
    iface_info.iface_name
  );
}
</script>

<template>
  <StatusBtn
    :status="status?.status"
    :disable_popover="disable_popover"
    @update:show="read_assigned_ips"
    @click="emit('click')"
  >
    <template #btn-icon>
      <n-icon>
        <NetworkEnterprise />
      </n-icon>
    </template>

    <template #popover-panel>
      <!-- {{ status?.data }} -->
      <n-data-table
        :columns="columns"
        :data="conver_to_show(assigned_ips)"
        :bordered="false"
      />
    </template>
  </StatusBtn>
</template>
