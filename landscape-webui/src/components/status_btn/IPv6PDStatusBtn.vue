<script setup lang="ts">
import { ZoneType } from "@/lib/service_ipconfig";
import { NetworkPublic, Cloud, NetworkEnterprise } from "@vicons/carbon";

import StatusBtn from "@/components/status_btn/StatusBtn.vue";
import { useIPv6PDStore } from "@/stores/status_ipv6pd";

const ipv6PDStore = useIPv6PDStore();

const iface_info = defineProps<{
  iface_name: string;
  zone: ZoneType;
}>();

const status = ipv6PDStore.GET_STATUS_BY_IFACE_NAME(iface_info.iface_name);

const emit = defineEmits(["click"]);
</script>

<template>
  <StatusBtn :status="status" @click="emit('click')">
    <template #btn-icon>
      <n-icon v-if="iface_info.zone == ZoneType.Wan">
        <NetworkPublic></NetworkPublic>
      </n-icon>
      <n-icon v-else-if="iface_info.zone == ZoneType.Lan">
        <NetworkEnterprise></NetworkEnterprise>
      </n-icon>
      <n-icon v-else>
        <Cloud></Cloud>
      </n-icon>
    </template>
  </StatusBtn>
</template>
