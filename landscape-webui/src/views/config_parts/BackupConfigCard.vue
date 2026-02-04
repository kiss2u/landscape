<script setup lang="ts">
import { get_init_config } from "@/api/sys/config";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

async function export_file() {
  await get_init_config();
}
</script>

<template>
  <n-card :title="t('config.backup_title')" segmented id="backup-config">
    <n-p>
      {{ t("config.backup_desc") }}
    </n-p>
    <n-button @click="export_file" type="info" ghost>
      {{ t("config.export_init") }}
    </n-button>
  </n-card>
</template>
