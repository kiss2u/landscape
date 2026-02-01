<script setup lang="ts">
import { computed, ref, watch } from "vue";

const ip = defineModel<string | undefined>("ip", { required: true });
const mask = defineModel<number | undefined>("mask");

interface Props {
  mask_max?: number;
}

const props = withDefaults(defineProps<Props>(), {
  mask_max: 32,
});

const ipParts = ref<[number, number, number, number]>([0, 0, 0, 0]);
watch(
  ip,
  (newX) => {
    const parts = (newX ?? "0.0.0.0")
      .split(".")
      .map((part) => Number(part) || 0);
    ipParts.value = [
      parts[0] ?? 0,
      parts[1] ?? 0,
      parts[2] ?? 0,
      parts[3] ?? 0,
    ];
  },
  { immediate: true },
);

const ip_parts_watch = computed(() => {
  return `${ipParts.value[0]}.${ipParts.value[1]}.${ipParts.value[2]}.${ipParts.value[3]}`;
});
watch(ip_parts_watch, (new_ip) => {
  ip.value = new_ip;
});
</script>

<template>
  <n-input-group>
    <n-input-number
      v-model:value="ipParts[0]"
      :show-button="false"
      min="0"
      max="255"
      placeholder=""
    />
    <n-input-group-label>.</n-input-group-label>
    <n-input-number
      v-model:value="ipParts[1]"
      :show-button="false"
      min="0"
      max="255"
      placeholder=""
    />
    <n-input-group-label>.</n-input-group-label>
    <n-input-number
      v-model:value="ipParts[2]"
      :show-button="false"
      min="0"
      max="255"
      placeholder=""
    />
    <n-input-group-label>.</n-input-group-label>
    <n-input-number
      v-model:value="ipParts[3]"
      :show-button="false"
      min="0"
      max="255"
      placeholder=""
    />
    <n-input-group-label v-if="mask !== undefined">/</n-input-group-label>
    <n-input-number
      v-if="mask !== undefined"
      min="0"
      :max="props.mask_max"
      v-model:value="mask"
      placeholder="mask"
    />
  </n-input-group>
</template>
