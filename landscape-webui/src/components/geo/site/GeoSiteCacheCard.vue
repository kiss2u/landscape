<script setup lang="ts">
import { delete_geo_site_config } from "@/api/geo/site";
import {
  GeoDomainConfig,
  GeoSiteConfig,
} from "@/rust_bindings/common/geo_site";
import { ref } from "vue";

const emit = defineEmits(["refresh"]);

interface Prop {
  geo_site: GeoDomainConfig;
}
const props = defineProps<Prop>();
const show_detail_modal = ref(false);
</script>
<template>
  <n-card :title="geo_site.key" size="small" style="flex: 1">
    <n-tag :bordered="false">
      {{ geo_site.name }}
    </n-tag>
    <template #header-extra>
      <n-flex>
        <n-button type="warning" secondary @click="show_detail_modal = true">
          详情
        </n-button>
      </n-flex>
      <GeoSiteDetailDrawer :geo_key="geo_site" v-model:show="show_detail_modal">
      </GeoSiteDetailDrawer>
    </template>
  </n-card>
</template>
