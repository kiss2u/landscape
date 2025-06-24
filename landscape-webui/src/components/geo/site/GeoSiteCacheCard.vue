<script setup lang="ts">
import { GeoDomainConfig } from "@/rust_bindings/common/geo_site";
import { ref } from "vue";
import { useFrontEndStore } from "@/stores/front_end_config";
import { mask_string } from "@/lib/common";

const frontEndStore = useFrontEndStore();
const emit = defineEmits(["refresh"]);

interface Prop {
  geo_site: GeoDomainConfig;
}
const props = defineProps<Prop>();
const show_detail_modal = ref(false);
</script>
<template>
  <n-card
    :title="
      frontEndStore.presentation_mode ? mask_string(geo_site.key) : geo_site.key
    "
    size="small"
    style="flex: 1"
  >
    <n-tag :bordered="false">
      {{
        frontEndStore.presentation_mode
          ? mask_string(geo_site.name)
          : geo_site.name
      }}
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
