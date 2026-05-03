<script setup lang="ts">
const model = defineModel<string>({ required: true });
defineProps<{ placeholder?: string }>();
</script>

<template>
  <n-input
    v-model:value="model"
    :placeholder="placeholder || 'e.g. 192.168.1.1 or tftp.example.com'"
    style="width: 100%"
  />
</template>
