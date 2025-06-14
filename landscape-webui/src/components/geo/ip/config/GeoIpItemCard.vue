<script setup lang="ts">
import { delete_geo_ip_config } from "@/api/geo/ip";
import { GeoIpSourceConfig } from "@/rust_bindings/common/geo_ip";
import { ref } from "vue";
import { useFrontEndStore } from "@/stores/front_end_config";
import { mask_string } from "@/lib/common";

const frontEndStore = useFrontEndStore();
const emit = defineEmits(["refresh"]);

interface Prop {
  geo_ip_source: GeoIpSourceConfig;
}
const props = defineProps<Prop>();
const show_edit_modal = ref(false);

async function del() {
  if (props.geo_ip_source.id) {
    await delete_geo_ip_config(props.geo_ip_source.id);
    emit("refresh");
  }
}
</script>
<template>
  <n-flex>
    <n-card :title="geo_ip_source.name" size="small">
      <!-- {{ geo_ip_source }} -->
      <n-descriptions bordered label-placement="top" :column="3">
        <n-descriptions-item label="名称">
          {{
            frontEndStore.presentation_mode
              ? mask_string(geo_ip_source.name)
              : geo_ip_source.name
          }}
        </n-descriptions-item>
        <n-descriptions-item label="启用">
          {{ geo_ip_source.enable }}
        </n-descriptions-item>
        <n-descriptions-item label="URL">
          {{
            frontEndStore.presentation_mode
              ? mask_string(geo_ip_source.url)
              : geo_ip_source.url
          }}
        </n-descriptions-item>
        <n-descriptions-item label="下次更新时间">
          {{ geo_ip_source.next_update_at }}
        </n-descriptions-item>
      </n-descriptions>
      <template #header-extra>
        <n-flex>
          <n-button type="warning" secondary @click="show_edit_modal = true">
            编辑
          </n-button>

          <n-popconfirm @positive-click="del()">
            <template #trigger>
              <n-button type="error" secondary @click=""> 删除 </n-button>
            </template>
            确定删除吗
          </n-popconfirm>
        </n-flex>
      </template>
    </n-card>
    <GeoIpEditModal
      :id="geo_ip_source.id"
      @refresh="emit('refresh')"
      v-model:show="show_edit_modal"
    ></GeoIpEditModal>
  </n-flex>
</template>
