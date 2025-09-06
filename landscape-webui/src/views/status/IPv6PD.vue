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
  <n-flex>
    <n-grid x-gap="12" y-gap="10" cols="1 600:2 1200:3 1600:3">
      <n-grid-item v-for="(config, iface_name) in infos" :key="iface_name">
        <IAPrefixInfoCard :config="config" :iface_name="iface_name" />
      </n-grid-item>
    </n-grid>
  </n-flex>
  <!-- {{ infos }} -->
</template>
