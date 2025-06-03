<script setup lang="ts">
import { get_geo_site_configs } from "@/api/geo/site";
import { GeoSiteConfig } from "@/rust_bindings/common/geo_site";
import { onMounted, ref } from "vue";

const emit = defineEmits(["refresh"]);

const show = defineModel<boolean>("show", { required: true });
const show_create_modal = ref(false);

const configs = ref<any>();
onMounted(async () => {
  await refresh();
});

async function refresh() {
  configs.value = await get_geo_site_configs();
}
</script>
<template>
  <n-drawer
    @after-enter="refresh"
    v-model:show="show"
    width="500px"
    placement="right"
  >
    <n-drawer-content title="Geo 配置来源" closable>
      <n-flex style="height: 100%" vertical>
        <n-button @click="show_create_modal = true">增加规则</n-button>

        <n-scrollbar>
          <n-flex vertical>
            <GeoSiteItemCard
              @refresh="refresh"
              v-for="rule in configs"
              :key="rule.index"
              :geo_site="rule"
            >
            </GeoSiteItemCard>
          </n-flex>
        </n-scrollbar>
      </n-flex>

      <GeoSiteEditModal
        :id="null"
        v-model:show="show_create_modal"
        @refresh="refresh"
      ></GeoSiteEditModal>
    </n-drawer-content>
  </n-drawer>
</template>
