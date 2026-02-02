<script setup lang="ts">
import { ref } from "vue";
import { useThemeVars } from "naive-ui";

const themeVars = useThemeVars();
</script>

<template>
  <n-flex vertical style="padding: 20px">
    <n-card title="DNS 指标监控">
      <n-empty description="DNS 指标开发中..." />
    </n-card>
  </n-flex>
</template>
