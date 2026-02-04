<script setup lang="ts">
import { useDnsConfigStore } from "@/stores/dns_config";
import { useMessage } from "naive-ui";

const dnsStore = useDnsConfigStore();
const message = useMessage();

async function handleSaveDns() {
  try {
    await dnsStore.saveDnsConfig();
    message.success("DNS 配置保存成功");
  } catch (e: any) {
    if (e.response?.status === 409) {
      message.error("DNS 配置冲突，请刷新后重试");
    } else {
      message.error("保存失败: " + e.message);
    }
  }
}
</script>

<template>
  <n-card title="DNS 全局配置" segmented id="dns-config">
    <template #header-extra>
      <n-button type="primary" @click="handleSaveDns"> 保存 DNS 配置 </n-button>
    </template>
    <n-form label-placement="left" label-width="120">
      <n-form-item label="缓存容量">
        <n-input-number
          v-model:value="dnsStore.cacheCapacity"
          :min="1024"
          :max="1048576"
          placeholder="4096"
          style="width: 200px"
        />
        <template #feedback> DNS 缓存允许保存的最大记录数 </template>
      </n-form-item>
      <n-form-item label="缓存 TTL (秒)">
        <n-input-number
          v-model:value="dnsStore.cacheTtl"
          :min="60"
          :max="2592000"
          placeholder="86400"
          style="width: 200px"
        />
        <template #feedback> DNS 缓存记录的最长保存时间 </template>
      </n-form-item>
    </n-form>
  </n-card>
</template>
