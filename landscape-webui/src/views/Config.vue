<script setup lang="ts">
import { get_init_config } from "@/api/sys/config";

async function export_file() {
  await get_init_config();
}
</script>

<template>
  <n-button @click="export_file">导出当前所有配置为 Init 文件</n-button>
</template>
