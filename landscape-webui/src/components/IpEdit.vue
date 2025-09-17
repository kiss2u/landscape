<script setup lang="ts">
import { isIP } from "is-ip";
const ip = defineModel<string | undefined>("ip", { required: true });
const mask = defineModel<number | undefined>("mask");

const rule = {
  trigger: ["input", "blur"],
  validator() {
    if (ip.value && !isIP(ip.value)) {
      return new Error("IP 格式不正确");
    }
  },
};
</script>

<template>
  <n-form-item
    style="flex: 1"
    :show-label="false"
    :show-feedback="false"
    :rule="rule"
  >
    <n-input-group>
      <n-input
        style="flex: 1"
        v-model:value="ip"
        placeholder="请输入 IPv4 或者 IPv6"
      />
      <n-input-group-label v-if="mask !== undefined">/</n-input-group-label>
      <n-input-number
        style="width: 90px"
        v-if="mask !== undefined"
        v-model:value="mask"
        placeholder="mask"
      />
    </n-input-group>
  </n-form-item>
</template>
