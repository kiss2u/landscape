<script setup lang="ts">
import { useI18n } from "vue-i18n";

import PacketDump from "@/components/PacketDump.vue";
import CPUUsage from "@/components/sysinfo/CPUUsage.vue";
import MemUsage from "@/components/sysinfo/MemUsage.vue";
import DnsStatusCard from "@/components/dns/DnsStatusCard.vue";
import SystemInfo from "@/components/sysinfo/SystemInfo.vue";
import NetFlow from "@/components/topology/NetFlow.vue";

import NetTopology from "@/components/topology_v2/LandscapeTopology.vue";

import { ref } from "vue";

const show_dump = ref(false);

const { t } = useI18n({ useScope: "global" });
</script>

<template>
  <n-flex vertical style="width: 100%">
    <n-grid x-gap="12" y-gap="12" cols="1 600:2 1200:4 1900:6">
      <n-gi style="display: flex; height: 280px" :span="1">
        <SystemInfo></SystemInfo>
      </n-gi>
      <n-gi style="display: flex; height: 280px" :span="1">
        <CPUUsage></CPUUsage>
      </n-gi>
      <n-gi style="display: flex; height: 280px" :span="1">
        <MemUsage></MemUsage>
      </n-gi>
      <n-gi style="display: flex; height: 280px" :span="1">
        <DnsStatusCard></DnsStatusCard>
      </n-gi>
    </n-grid>
    <n-divider style="margin: 0px 0" title-placement="left">
      {{ t("topology_divider") }}
    </n-divider>
    <!-- <NetTopology
      style="flex: 1; min-height: 550px; min-width: 100%"
    ></NetTopology> -->
    <NetFlow style="flex: 1; min-height: 550px; min-width: 100%"></NetFlow>
    <!-- <PacketDump v-model="show_dump"></PacketDump> -->
    <!-- <n-button @click="show_dump = true">Show DUMP</n-button> -->
  </n-flex>
</template>
