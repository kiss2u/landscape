<script setup lang="ts">
import { get_geo_cache_key, refresh_geo_cache_key } from "@/api/geo/site";
import { onMounted, ref } from "vue";

const rules = ref<any>([]);

onMounted(async () => {
  await refresh();
});

async function refresh() {
  rules.value = await get_geo_cache_key();
}

async function refresh_cache() {
  await refresh_geo_cache_key();
  await refresh();
}

const show_geo_drawer_modal = ref(false);
</script>
<template>
  <n-layout :native-scrollbar="false" content-style="padding: 10px;">
    <n-flex vertical>
      <n-flex>
        <n-button @click="show_geo_drawer_modal = true">Geo 配置</n-button>
        <n-button @click="refresh_cache">手动触发更新</n-button>
        <GeoSiteDrawer v-model:show="show_geo_drawer_modal"></GeoSiteDrawer>
      </n-flex>
      <!-- {{ rules }} -->
      <n-grid x-gap="12" y-gap="10" cols="1 600:2 900:3 1200:4 1600:5">
        <n-grid-item
          v-for="rule in rules"
          :key="rule.index"
          style="display: flex"
        >
          <GeoSiteCacheCard :geo_site="rule"></GeoSiteCacheCard>
        </n-grid-item>
      </n-grid>
    </n-flex>
  </n-layout>
</template>
