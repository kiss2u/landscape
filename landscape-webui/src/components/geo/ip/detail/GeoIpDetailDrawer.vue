<script setup lang="ts">
import { get_geo_ip_cache_detail } from "@/api/geo/ip";
import { GeoConfigKey } from "@/rust_bindings/common/geo";
import { GeoIpConfig } from "@/rust_bindings/common/geo_ip";
import { onMounted, ref } from "vue";

const key = defineModel<GeoConfigKey>("geo_key", {
  required: true,
});
const show = defineModel<boolean>("show", { required: true });

const config = ref<GeoIpConfig>();

async function refresh() {
  config.value = await get_geo_ip_cache_detail(key.value);
}
</script>
<template>
  <n-drawer
    @after-enter="refresh"
    v-model:show="show"
    width="500px"
    placement="right"
  >
    <n-drawer-content title="规则细节" closable>
      <n-scrollbar v-if="config">
        <n-flex>
          <n-card
            v-for="(rule, index) in config.values"
            :key="index"
            :title="`${rule.ip}/${rule.prefix}`"
            size="small"
          >
          </n-card>
        </n-flex>
      </n-scrollbar>
    </n-drawer-content>
  </n-drawer>
</template>
