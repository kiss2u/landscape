<script setup lang="ts">
import { NCard } from "naive-ui";
import { onMounted, ref } from "vue";

import { ServiceStatus } from "@/lib/services";

const current_status = ref<ServiceStatus>(new ServiceStatus());

const show_rule_drawer = ref(false);
onMounted(async () => {
  // current_status.value = await get_dns_status();
});

async function start_dns() {
  // current_status.value = await start_dns_service(53);
  // await pollDnsStatus();
}

async function stop_dns() {
  // current_status.value = await stop_dns_service();
  // await pollDnsStatus();
}
</script>
<template>
  <n-card title="防火墙">
    <template #header-extra>
      <n-button @click="show_rule_drawer = true">规则配置</n-button>
      <n-button @click="start_dns" v-if="current_status.t == 'stop'">
        开启
      </n-button>
      <n-button v-else @click="stop_dns">关闭</n-button>
    </template>
    {{ current_status }}
    <!-- <template #footer> #footer </template>
    <template #action> #action </template> -->
    <!-- <DnsRuleDrawer v-model:show="show_rule_drawer" /> -->
  </n-card>
</template>
