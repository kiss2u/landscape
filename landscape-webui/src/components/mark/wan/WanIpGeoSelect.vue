<script setup lang="ts">
import { get_geo_ip_configs, search_geo_ip_cache } from "@/api/geo/ip";
import { GeoConfigKey } from "@/rust_bindings/common/geo";
import { GeoIpConfig, GeoIpSourceConfig } from "@/rust_bindings/common/geo_ip";
import { computed, onMounted, ref } from "vue";

const key = defineModel<string | null>("geo_key", {
  required: true,
});
const name = defineModel<string | null>("geo_name", {
  required: true,
});
const emit = defineEmits(["refresh"]);

const loading_name = ref(false);
const loading_key = ref(false);

const configs = ref<GeoIpSourceConfig[]>();
async function typing_name_key(query?: string) {
  try {
    loading_name.value = true;
    configs.value = await get_geo_ip_configs(query);
  } finally {
    loading_name.value = false;
  }
}
const geo_name_options = computed(() => {
  let result = [];
  if (configs.value) {
    for (const config of configs.value) {
      result.push({
        label: config.name,
        value: config.name,
      });
    }
  }
  return result;
});

async function typing_key(query: string) {
  try {
    loading_key.value = true;
    keys.value = await search_geo_ip_cache({
      name: name.value,
      key: query,
    });
  } finally {
    loading_key.value = false;
  }
}

const keys = ref<GeoConfigKey[]>();
const geo_key_options = computed(() => {
  let result = [];
  let show_name = name.value == "" || name.value == null;
  console.log(name.value);
  if (keys.value) {
    for (const each_key of keys.value) {
      result.push({
        label: show_name ? `${each_key.name}-${each_key.key}` : each_key.key,
        value: each_key.key,
        data: each_key,
      });
    }
  }
  return result;
});

function select_key(value: string, option: any) {
  key.value = value;
  name.value = option.data.name;
}

async function update_key() {
  await typing_key("");
  emit("refresh");
}

async function show_keys() {
  await typing_key("");
}

async function show_names() {
  await typing_name_key("");
}
</script>
<template>
  <n-input-group>
    <n-select
      :style="{ width: '33%' }"
      v-model:value="name"
      filterable
      placeholder="选择 geo 名称"
      :options="geo_name_options"
      :loading="loading_name"
      clearable
      remote
      @update:show="show_names"
      @update:value="update_key"
      @search="typing_name_key"
    />
    <n-select
      v-model:value="key"
      filterable
      placeholder="筛选key"
      :options="geo_key_options"
      :loading="loading_key"
      clearable
      remote
      @update:show="show_keys"
      @update:value="select_key"
      @search="typing_key"
    />
  </n-input-group>
</template>
