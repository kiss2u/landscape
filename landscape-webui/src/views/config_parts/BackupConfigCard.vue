<script setup lang="ts">
import { get_init_config } from "@/api/sys/config";

async function export_file() {
  await get_init_config();
}
</script>

<template>
  <n-card title="备份与导出" segmented id="backup-config">
    <n-p>
      你可以将当前路由器的所有配置（不含 Docker
      相关）导出为一个初始化文件，用于快速恢复或迁移。
    </n-p>
    <n-button @click="export_file" type="info" ghost>
      导出当前所有配置为 Init 文件
    </n-button>
  </n-card>
</template>
