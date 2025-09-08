<script lang="ts" setup>
import { get_dhcp_v4_assigned_ips } from "@/api/service_dhcp_v4";
import { DHCPv4OfferInfo } from "@/rust_bindings/common/dhcp_v4_server";
import { computed, onMounted, ref } from "vue";

onMounted(async () => {
  await get_info();
});

const infos = ref<Map<String, DHCPv4OfferInfo | null>>(new Map());
async function get_info() {
  infos.value = await get_dhcp_v4_assigned_ips();
}
</script>

<template>
  <n-flex vertical style="flex: 1">
    <n-flex>
      <n-button @click="get_info">刷新</n-button>
    </n-flex>
    <!-- {{ infos }} -->
    <n-flex v-if="infos.size > 0">
      <AssignedIpTable
        @refresh="get_info"
        v-for="([iface_name, config], index) in infos"
        :key="index"
        :iface_name="iface_name"
        :info="config"
      ></AssignedIpTable>
    </n-flex>
    <n-empty style="flex: 1" v-else></n-empty
  ></n-flex>

  <!-- {{ infos }} -->
</template>
