<script setup lang="ts">
import { onMounted, ref } from "vue";
import { usePreferenceStore } from "@/stores/preference";
import { useMetricConfigStore } from "@/stores/metric_config";
import { useDnsConfigStore } from "@/stores/dns_config";
import { useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";

import UIConfigCard from "@/views/config_parts/UIConfigCard.vue";
import DNSConfigCard from "@/views/config_parts/DNSConfigCard.vue";
import MetricConfigCard from "@/views/config_parts/MetricConfigCard.vue";
import BackupConfigCard from "@/views/config_parts/BackupConfigCard.vue";

const { t } = useI18n();
const prefStore = usePreferenceStore();
const metricStore = useMetricConfigStore();
const dnsStore = useDnsConfigStore();
const message = useMessage();
const loading = ref(false);

const scrollTarget = () => document.querySelector(".main-body");

onMounted(async () => {
  loading.value = true;
  try {
    await Promise.all([
      prefStore.loadPreferenceForEdit(),
      metricStore.loadMetricConfig(),
      dnsStore.loadDnsConfig(),
    ]);
  } catch (e) {
    message.error("加载配置失败");
    console.error(e);
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="config-container">
    <n-row gutter="48">
      <n-col :span="18">
        <n-space vertical size="large">
          <UIConfigCard />
          <DNSConfigCard />
          <MetricConfigCard />
          <BackupConfigCard />
          <div style="height: 400px"></div>
        </n-space>
      </n-col>
      <n-col :span="6">
        <div class="anchor-wrapper">
          <!-- 使用 n-anchor 自带的 affix 功能，并设置合适的顶部距离 -->
          <n-anchor
            affix
            :top="60"
            :offset-top="60"
            :bound="24"
            :ignore-gap="true"
            listen-to=".main-body"
            style="width: 200px"
          >
            <n-card
              title="配置目录"
              size="small"
              :segmented="{ content: true }"
            >
              <n-anchor-link title="偏好设置" href="#ui-config" />
              <n-anchor-link title="DNS 配置" href="#dns-config" />
              <n-anchor-link title="指标配置" href="#metric-config" />
              <n-anchor-link title="备份导出" href="#backup-config" />
            </n-card>
          </n-anchor>
        </div>
      </n-col>
    </n-row>
  </div>
</template>

<style scoped>
.config-container {
  padding: 24px;
  width: 100%;
}

.anchor-wrapper {
  position: relative;
  height: 100%;
}

:deep(.n-anchor-link) {
  font-size: 14px;
  padding: 8px 16px;
}

:deep(.n-card-header) {
  padding: 12px 16px !important;
}
</style>
