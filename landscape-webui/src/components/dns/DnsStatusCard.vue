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
import { useDnsStore } from "@/stores/status_dns";

const dnsStore = useDnsStore();

const show_rule_drawer = ref(false);

async function start_dns() {
  await start_dns_service(53);
}

async function stop_dns() {
  await stop_dns_service();
}
</script>
<template>
  <n-card title="DNS 服务">
    <template #header-extra>
      <n-flex>
        <n-button @click="show_rule_drawer = true">域名解析规则</n-button>
        <n-button @click="start_dns" v-if="dnsStore.is_down"> 开启 </n-button>
        <n-button v-else @click="stop_dns">关闭</n-button>
      </n-flex>
    </template>
    {{ dnsStore.dns_status }}
    <!-- <template #footer> #footer </template>
    <template #action> #action </template> -->
    <DnsRuleDrawer v-model:show="show_rule_drawer" />
  </n-card>
</template>
