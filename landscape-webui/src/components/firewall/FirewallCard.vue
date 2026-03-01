<script setup lang="ts">
import { useI18n } from "vue-i18n";
const { t } = useI18n();
</script>
<template>
  <n-card :title="t('firewall.card.title')" content-style="display: flex;">
    <n-flex justify="center" align="center" style="flex: 1">
      <n-empty description="TODO"> </n-empty>
    </n-flex>
  </n-card>
</template>
