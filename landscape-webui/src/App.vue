<script setup lang="ts">
import { darkTheme } from "naive-ui";
import { dateZhCN, zhCN } from "naive-ui";
</script>

<template>
  <n-config-provider
    :locale="zhCN"
    :date-locale="dateZhCN"
    :theme="darkTheme"
    style="display: flex; flex: 1"
    :theme-overrides="{ common: { fontWeightStrong: '600' } }"
  >
    <n-message-provider>
      <n-notification-provider>
        <RouterView />
      </n-notification-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
/* .main-body {
  align-items: center;
  width: 100%;
  display: flex;
  justify-items: center;
} */
</style>
