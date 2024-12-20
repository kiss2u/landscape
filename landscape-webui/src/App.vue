<script setup lang="ts">
import PacketDump from "@/components/PacketDump.vue";
import CPUUsage from "@/components/sysinfo/CPUUsage.vue";
import MemUsage from "@/components/sysinfo/MemUsage.vue";
import DnsStatusCard from "@/components/dns/DnsStatusCard.vue";
import FireWallStatusCard from "@/components/firewall/FireWallStatusCard.vue";
import IntervalFetch from "./components/head/IntervalFetch.vue";

import NetFlow from "@/components/flow/NetFlow.vue";
import { computed, onMounted, ref } from "vue";

const show_dump = ref(false);
</script>

<template>
  <n-config-provider :theme-overrides="{ common: { fontWeightStrong: '600' } }">
    <n-layout position="absolute" style="">
      <n-layout-header style="height: 24px">
        <n-flex justify="space-between">
          <n-flex>Landscape</n-flex>
          <n-flex><IntervalFetch /></n-flex>
        </n-flex>
      </n-layout-header>
      <!-- <n-layout has-sider>
    <n-layout-sider content-style="padding: 24px;">
      海淀桥
    </n-layout-sider>
    <n-layout-content content-style="padding: 24px;">
      平山道
    </n-layout-content>
  </n-layout> -->
      <n-layout-content
        :native-scrollbar="false"
        position="absolute"
        style="top: 24px; bottom: 24px"
        content-class="main-body"
        content-style="padding: 10px;"
      >
        <n-flex vertical>
          <n-grid x-gap="12" y-gap="12" :cols="4">
            <n-gi :span="4">
              <n-flex align="center" justify="center" style="flex: 1">
                <CPUUsage> </CPUUsage>
                <MemUsage> </MemUsage>
              </n-flex>
            </n-gi>
            <n-gi :span="4">
              <DnsStatusCard></DnsStatusCard>
            </n-gi>
            <!-- <n-gi :span="2">
              <FireWallStatusCard></FireWallStatusCard>
            </n-gi> -->
          </n-grid>

          <NetFlow></NetFlow>
        </n-flex>

        <!-- <n-button @click="show_dump = true">Show DUMP</n-button> -->

        <PacketDump v-model="show_dump"></PacketDump>
      </n-layout-content>
      <n-layout-footer position="absolute" style="height: 24px">
        // some info
      </n-layout-footer>
    </n-layout>
  </n-config-provider>
</template>

<style>
.main-body {
  /* align-items: center;
  width: 100%;
  display: flex;
  justify-items: center; */
}
</style>
