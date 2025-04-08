<script setup lang="ts">
import { useI18n } from "vue-i18n";

import PacketDump from "@/components/PacketDump.vue";
import CPUUsage from "@/components/sysinfo/CPUUsage.vue";
import MemUsage from "@/components/sysinfo/MemUsage.vue";
import DnsStatusCard from "@/components/dns/DnsStatusCard.vue";
import DockerStatusCard from "@/components/docker/DockerStatusCard.vue";
import DockerAllContainer from "@/components/docker/DockerAllContainer.vue";
import FirewallCard from "@/components/firewall/FirewallCard.vue";
import SystemInfo from "@/components/sysinfo/SystemInfo.vue";

import NetFlow from "@/components/flow/NetFlow.vue";

import { ref } from "vue";
import MarkRuleCard from "@/components/mark/MarkRuleCard.vue";

const show_dump = ref(false);

const { t } = useI18n({ useScope: "global" });
</script>

<template>
  <n-layout :native-scrollbar="false" content-style="padding: 10px;">
    <n-flex vertical>
      <n-grid x-gap="12" y-gap="12" cols="1 600:2 1200:4 1900:6">
        <n-gi style="display: flex; height: 330px" :span="1">
          <SystemInfo></SystemInfo>
        </n-gi>
        <n-gi style="display: flex; height: 330px" :span="1">
          <CPUUsage></CPUUsage>
        </n-gi>
        <n-gi style="display: flex; height: 330px" :span="1">
          <MemUsage></MemUsage>
        </n-gi>
        <n-gi style="display: flex; height: 330px" :span="1">
          <DnsStatusCard></DnsStatusCard>
        </n-gi>
        <n-gi style="display: flex; height: 330px" :span="1">
          <DockerStatusCard></DockerStatusCard>
        </n-gi>
        <n-gi style="display: flex; height: 330px" :span="1">
          <n-flex
            ><MarkRuleCard></MarkRuleCard>
            <FirewallCard></FirewallCard>
          </n-flex>
        </n-gi>
      </n-grid>
      <n-divider style="margin: 0px 0" title-placement="left">
        {{ t("docker_divider") }}
      </n-divider>
      <DockerAllContainer></DockerAllContainer>
      <n-divider style="margin: 0px 0" title-placement="left">
        {{ t("topology_divider") }}
      </n-divider>
      <NetFlow></NetFlow>
    </n-flex>

    <!-- <n-button @click="show_dump = true">Show DUMP</n-button> -->

    <PacketDump v-model="show_dump"></PacketDump>
  </n-layout>
</template>
