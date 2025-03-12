<script setup lang="ts">
import { ZoneType } from "@/lib/service_ipconfig";
import { Ad } from "@vicons/tabler";

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
      <Ad></Ad>
    </template>
  </StatusBtn>
</template>
