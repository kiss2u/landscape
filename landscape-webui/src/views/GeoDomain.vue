<script setup lang="ts">
import { refresh_geo_cache_key, search_geo_site_cache } from "@/api/geo/site";
import { QueryGeoKey } from "@/rust_bindings/common/geo";
import { onMounted, ref } from "vue";

const rules = ref<any>([]);

onMounted(async () => {
  await refresh();
});

const filter = ref<QueryGeoKey>({
  name: null,
  key: null,
});

async function refresh() {
  rules.value = await search_geo_site_cache(filter.value);
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
      <n-flex :wrap="false">
        <!-- {{ filter }} -->
        <n-button @click="show_geo_drawer_modal = true">Geo 配置</n-button>
        <n-button @click="refresh_cache">手动触发更新</n-button>
        <GeoSiteNameSelect
          v-model:name="filter.name"
          @refresh="refresh"
        ></GeoSiteNameSelect>
        <GeoSiteKeySelect
          v-model:geo_key="filter.key"
          v-model:name="filter.name"
          @refresh="refresh"
        ></GeoSiteKeySelect>
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

    <GeoSiteDrawer v-model:show="show_geo_drawer_modal"></GeoSiteDrawer>
  </n-layout>
</template>
