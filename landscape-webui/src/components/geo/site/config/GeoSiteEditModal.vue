<script setup lang="ts">
import { get_geo_site_config, push_geo_site_config } from "@/api/geo/site";
import { GeoSiteConfig } from "@/rust_bindings/common/geo_site";
import { computed, ref } from "vue";

const emit = defineEmits(["refresh"]);

const show = defineModel<boolean>("show", { required: true });
interface Prop {
  id: string | null;
}
const props = defineProps<Prop>();
const commit_spin = ref(false);

const rule = ref<GeoSiteConfig>();
const rule_json = ref<string>("");

async function enter() {
  if (props.id !== null) {
    rule.value = await get_geo_site_config(props.id);
  } else {
    rule.value = {
      id: null,
      update_at: new Date().getTime(),
      url: "",
      name: "",
      enable: true,
      next_update_at: 0,
      geo_keys: [],
    };
  }
  rule_json.value = JSON.stringify(rule.value);
}

const isModified = computed(() => {
  return JSON.stringify(rule.value) !== rule_json.value;
});

async function saveRule() {
  if (rule.value) {
    try {
      commit_spin.value = true;
      await push_geo_site_config(rule.value);
      show.value = false;
      emit("refresh");
    } finally {
      commit_spin.value = false;
    }
  }
}
</script>
<template>
  <n-modal
    v-model:show="show"
    style="width: 600px"
    preset="card"
    title="编辑 Geo Site"
    size="small"
    :bordered="false"
    @after-enter="enter"
  >
    <!-- {{ rule }}
    {{ rule_json }} -->
    <n-form v-if="rule" style="flex: 1" ref="formRef" :model="rule" :cols="5">
      <n-grid :cols="5">
        <n-form-item-gi label="启用" :offset="0" :span="1">
          <n-switch v-model:value="rule.enable">
            <template #checked> 启用 </template>
            <template #unchecked> 禁用 </template>
          </n-switch>
        </n-form-item-gi>
        <n-form-item-gi label="下载 URL" :span="5">
          <n-input v-model:value="rule.url" clearable />
        </n-form-item-gi>

        <n-form-item-gi label="名称 (与其他配置区分， 需要唯一)" :span="5">
          <n-input v-model:value="rule.name" clearable />
        </n-form-item-gi>
      </n-grid>
    </n-form>
    <template #footer>
      <n-flex justify="space-between">
        <n-button @click="show = false">取消</n-button>
        <n-button
          :loading="commit_spin"
          @click="saveRule"
          :disabled="!isModified"
        >
          保存
        </n-button>
      </n-flex></template
    >
  </n-modal>
</template>
