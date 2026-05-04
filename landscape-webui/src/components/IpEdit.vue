<script setup lang="ts">
import { computed } from "vue";
import { isIP, isIPv4, isIPv6 } from "is-ip";
const ip = defineModel<string | undefined>("ip", { required: true });
const mask = defineModel<number | undefined>("mask");

interface Props {
  mask_max?: number;
  ip_version?: 4 | 6;
}

const props = defineProps<Props>();

const placeholder = computed(() => {
  if (props.ip_version === 4) return "请输入 IPv4";
  if (props.ip_version === 6) return "请输入 IPv6";
  return "请输入 IPv4 或者 IPv6";
});

function is_valid_ip(value: string) {
  if (props.ip_version === 4) return isIPv4(value);
  if (props.ip_version === 6) return isIPv6(value);
  return isIP(value);
}

const rule = {
  trigger: ["input", "blur"],
  validator() {
    if (ip.value && !is_valid_ip(ip.value)) {
      return new Error("IP 格式不正确");
    }
  },
};
</script>

<template>
  <n-form-item
    style="flex: 1; width: 100%"
    :show-label="false"
    :show-feedback="false"
    :rule="rule"
  >
    <n-input-group style="width: 100%">
      <n-input
        style="flex: 1; min-width: 0"
        v-model:value="ip"
        :placeholder="placeholder"
      />
      <n-input-group-label v-if="mask !== undefined">/</n-input-group-label>
      <n-input-number
        style="width: 90px"
        v-if="mask !== undefined"
        v-model:value="mask"
        :min="0"
        :max="props.mask_max"
        placeholder="mask"
      />
    </n-input-group>
  </n-form-item>
</template>
