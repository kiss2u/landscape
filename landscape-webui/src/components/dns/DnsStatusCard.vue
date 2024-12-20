<script setup lang="ts">
import { NCard } from "naive-ui";
import { onMounted, ref } from "vue";

import {
  get_dns_rule,
  get_dns_status,
  start_dns_service,
  stop_dns_service,
} from "@/api/dns_service";
import { ServiceStatus } from "@/lib/services";
import DnsRuleDrawer from "@/components/dns/DnsRuleDrawer.vue";

const current_status = ref<ServiceStatus>(new ServiceStatus());

const show_rule_drawer = ref(false);
onMounted(async () => {
  current_status.value = await get_dns_status();
});

async function start_dns() {
  current_status.value = await start_dns_service(53);

  await pollDnsStatus();
}

async function stop_dns() {
  current_status.value = await stop_dns_service();

  await pollDnsStatus();
}

async function pollDnsStatus() {
  while (true) {
    current_status.value = await get_dns_status();

    if (
      current_status.value.t === "running" ||
      current_status.value.t === "stop"
    ) {
      break; // 满足条件时退出循环
    }

    // 设置一个合适的轮询间隔，避免过于频繁的请求
    await new Promise((resolve) => setTimeout(resolve, 1000)); // 1秒间隔
  }
}
</script>
<template>
  <n-card title="DNS 服务">
    <template #header-extra>
      <n-button @click="show_rule_drawer = true">域名解析规则</n-button>
      <n-button @click="start_dns" v-if="current_status.t == 'stop'">
        开启
      </n-button>
      <n-button v-else @click="stop_dns">关闭</n-button>
    </template>
    {{ current_status }}
    <!-- <template #footer> #footer </template>
    <template #action> #action </template> -->
    <DnsRuleDrawer v-model:show="show_rule_drawer" />
  </n-card>
</template>
