<script setup lang="ts">
import { computed } from "vue";
import type { FlowEntryRule } from "@landscape-router/types/api/schemas";
import { useEnrolledDeviceStore } from "@/stores/enrolled_device";

interface Prop {
  rule: FlowEntryRule;
}

const enrolledDeviceStore = useEnrolledDeviceStore();
const props = defineProps<Prop>();

const tagType = computed(() => {
  switch (props.rule.mode.t) {
    case "ip":
      return "info";
    case "device":
      return "success";
    case "mac":
      return "warning";
    default:
      return "default";
  }
});

const label = computed(() => {
  switch (props.rule.mode.t) {
    case "ip":
      return enrolledDeviceStore.GET_NAME_WITH_FALLBACK(
        props.rule.mode.ip,
        `${props.rule.mode.ip}/${props.rule.mode.prefix_len}`,
      );
    case "device":
      return enrolledDeviceStore.GET_DISPLAY_NAME_BY_ID(
        props.rule.mode.device_id,
      );
    case "mac":
      return enrolledDeviceStore.GET_NAME_WITH_FALLBACK(
        props.rule.mode.mac_addr,
      );
    default:
      return "";
  }
});
</script>

<template>
  <n-tag :bordered="false" :type="tagType">
    {{ label }}
  </n-tag>
</template>
