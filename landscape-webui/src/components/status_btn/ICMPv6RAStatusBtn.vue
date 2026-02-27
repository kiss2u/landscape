<script setup lang="ts">
import { ZoneType } from "@/lib/service_ipconfig";
import { Ad } from "@vicons/tabler";

import StatusBtn from "@/components/status_btn/StatusBtn.vue";
import { useLanIPv6Store } from "@/stores/status_lan_ipv6";
import { IfaceZoneType } from "@landscape-router/types/api/schemas";

const icmpv6PDStore = useLanIPv6Store();

const iface_info = defineProps<{
  iface_name: string;
  zone: IfaceZoneType;
}>();

const status = icmpv6PDStore.GET_STATUS_BY_IFACE_NAME(iface_info.iface_name);

const emit = defineEmits(["click"]);
</script>

<template>
  <StatusBtn :status="status" @click="emit('click')">
    <template #btn-icon>
      <Ad></Ad>
    </template>
  </StatusBtn>
</template>
