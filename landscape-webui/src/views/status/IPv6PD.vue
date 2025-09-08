<script lang="ts" setup>
import { get_current_ip_prefix_info } from "@/api/service_ipv6pd";
import { LDIAPrefix } from "@/rust_bindings/common/ipv6_pd";
import { computed, onMounted, ref } from "vue";

onMounted(async () => {
  await get_info();
});

const infos = ref<Map<String, LDIAPrefix | null>>(new Map());
async function get_info() {
  infos.value = await get_current_ip_prefix_info();
}
</script>

<template>
  <n-flex vertical style="flex: 1">
    <n-flex>
      <n-button @click="get_info">刷新</n-button>
    </n-flex>
    <n-flex v-if="infos.size > 0">
      <n-grid x-gap="12" y-gap="10" cols="1 600:2 1200:3 1600:3">
        <n-grid-item
          v-for="([iface_name, config], index) in infos"
          :key="index"
        >
          <IAPrefixInfoCard :config="config" :iface_name="iface_name" />
        </n-grid-item>
      </n-grid>
    </n-flex>
    <n-empty style="flex: 1" v-else></n-empty
  ></n-flex>

  <!-- {{ infos }} -->
</template>
