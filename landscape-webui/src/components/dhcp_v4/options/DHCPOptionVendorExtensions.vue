<script setup lang="ts">
const model = defineModel<string>({ required: true });
defineProps<{ placeholder?: string }>();
</script>

<template>
  <n-input
    v-model:value="model"
    :placeholder="placeholder || 'e.g. ff0001'"
    maxlength="510"
    :status="
      !model || !/^[\da-fA-F]+$/.test(model) || model.length % 2 !== 0
        ? 'error'
        : undefined
    "
    style="width: 100%"
  />
</template>
