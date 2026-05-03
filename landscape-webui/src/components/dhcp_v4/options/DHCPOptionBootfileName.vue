<script setup lang="ts">
const model = defineModel<string>({ required: true });
defineProps<{ placeholder?: string }>();
</script>

<template>
  <n-input
    v-model:value="model"
    :placeholder="placeholder || 'e.g. undionly.kpxe or boot/pxelinux.0'"
    style="width: 100%"
  />
</template>
