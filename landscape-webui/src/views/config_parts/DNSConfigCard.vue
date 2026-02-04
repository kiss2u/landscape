<script setup lang="ts">
import { useDnsConfigStore } from "@/stores/dns_config";
import { useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";

const dnsStore = useDnsConfigStore();
const message = useMessage();
const { t } = useI18n();

async function handleSaveDns() {
  try {
    await dnsStore.saveDnsConfig();
    message.success(t("config.save_success"));
  } catch (e: any) {
    if (e.response?.status === 409) {
      message.error(t("config.conflict"));
    } else {
      message.error(t("config.save_failed") + ": " + e.message);
    }
  }
}
</script>

<template>
  <n-card :title="t('config.dns_title')" segmented id="dns-config">
    <template #header-extra>
      <n-button type="primary" @click="handleSaveDns">
        {{ t("config.save_dns") }}
      </n-button>
    </template>
    <n-form label-placement="left" label-width="120">
      <n-form-item :label="t('config.cache_capacity')">
        <n-input-number
          v-model:value="dnsStore.cacheCapacity"
          :min="1024"
          :max="1048576"
          placeholder="4096"
          style="width: 200px"
        />
        <template #feedback> {{ t("config.cache_capacity_desc") }} </template>
      </n-form-item>
      <n-form-item :label="t('config.cache_ttl')">
        <n-input-number
          v-model:value="dnsStore.cacheTtl"
          :min="60"
          :max="2592000"
          placeholder="86400"
          style="width: 200px"
        />
        <template #feedback> {{ t("config.cache_ttl_desc") }} </template>
      </n-form-item>
    </n-form>
  </n-card>
</template>
