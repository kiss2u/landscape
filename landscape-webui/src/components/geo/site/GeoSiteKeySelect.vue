<script setup lang="ts">
import { delete_geo_site_config, search_geo_site_cache } from "@/api/geo/site";
import { GeoConfigKey } from "@/rust_bindings/common/geo";
import { computed, onMounted, ref } from "vue";

const geo_key = defineModel<string | null>("geo_key", { required: true });
const name = defineModel<string | null>("name", { required: true });

const loading = ref(false);

const keys = ref<GeoConfigKey[]>();
onMounted(async () => {
  await typing_key("");
});

async function typing_key(query: string) {
  try {
    loading.value = true;
    keys.value = await search_geo_site_cache({
      name: name.value,
      key: query,
    });
  } finally {
    loading.value = false;
  }
}

const emit = defineEmits(["refresh"]);

const geo_key_options = computed(() => {
  let result = [];
  if (keys.value) {
    for (const each_key of keys.value) {
      result.push({
        label: `${each_key.name}-${each_key.key}`,
        value: each_key.key,
        data: each_key,
      });
    }
  }
  return result;
});

function select_key(value: string, option: any) {
  geo_key.value = value;
  name.value = option.data.name;
  emit("refresh");
}

function clear() {
  geo_key.value = null;
  emit("refresh");
}
</script>
<template>
  <n-select
    v-model:value="geo_key"
    filterable
    placeholder="筛选key"
    :options="geo_key_options"
    :loading="loading"
    clearable
    remote
    @update:value="select_key"
    @clear="clear"
    @search="typing_key"
  />
</template>
