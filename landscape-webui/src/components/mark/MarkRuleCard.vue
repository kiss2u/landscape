<script setup lang="ts">
import { ref } from "vue";
import LanIpRuleDrawer from "./lan/LanIpRuleDrawer.vue";

const show_lan_ip_rules = ref<boolean>(false);
const show_wan_ip_rules = ref<boolean>(false);
</script>
<template>
  <n-card title="IP 标记配置" content-style="display: flex;">
    <template #header-extra>
      <n-flex>
        <n-button
          :focusable="false"
          size="small"
          @click="show_lan_ip_rules = true"
        >
          LAN
        </n-button>
        <n-button
          :focusable="false"
          size="small"
          @click="show_wan_ip_rules = true"
        >
          WAN
        </n-button>
      </n-flex>
    </template>
    <n-flex justify="center" align="center" style="flex: 1">
      <n-empty description="TODO"> </n-empty>
    </n-flex>
    <!-- <template #footer> #footer </template>
    <template #action> #action </template> -->
    <LanIpRuleDrawer v-model:show="show_lan_ip_rules" />
    <WanIpRuleDrawer v-model:show="show_wan_ip_rules" />
  </n-card>
</template>
