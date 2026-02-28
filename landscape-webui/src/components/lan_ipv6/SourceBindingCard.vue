<script setup lang="ts">
import type { LanIPv6SourceConfig } from "@landscape-router/types/api/schemas";
import { computed, ref } from "vue";
import { Edit, Delete } from "@vicons/carbon";
import { useI18n } from "vue-i18n";
import SourceBindingModal from "./SourceBindingModal.vue";

const { t } = useI18n({ useScope: "global" });

const props = defineProps<{
  source: LanIPv6SourceConfig;
  allowedServiceKinds?: ("ra" | "na" | "pd")[];
}>();

const tag_type = computed(() => {
  switch (props.source.t) {
    case "ra_static":
    case "na_static":
    case "pd_static":
      return "default";
    default:
      return "info";
  }
});

const display_text = computed(() => {
  const src = props.source;
  switch (src.t) {
    case "ra_static":
      return `${src.base_prefix} @ ${src.pool_index}`;
    case "ra_pd":
      return `${src.depend_iface} @ ${src.pool_index}`;
    case "na_static":
      return `${src.base_prefix} @ ${src.pool_index}`;
    case "na_pd":
      return `${src.depend_iface} @ ${src.pool_index}`;
    case "pd_static":
      return `${src.base_prefix}/${src.base_prefix_len} @ ${src.pool_index}/${src.pool_len}`;
    case "pd_pd":
      return `${src.depend_iface} @ ${src.pool_index}/${src.pool_len}`;
  }
});

const kind_label = computed(() => {
  switch (props.source.t) {
    case "ra_static":
    case "ra_pd":
      return "RA";
    case "na_static":
    case "na_pd":
      return "NA";
    case "pd_static":
    case "pd_pd":
      return "PD";
  }
});

const emit = defineEmits(["delete", "commit"]);

function emit_delete() {
  emit("delete");
}

function emit_commit(source: LanIPv6SourceConfig) {
  emit("commit", source);
}

const show_edit = ref(false);
</script>
<template>
  <n-tag :type="tag_type" :bordered="false">
    <n-text style="font-size: 11px; margin-right: 4px" depth="3">
      [{{ kind_label }}]
    </n-text>
    <span>{{ display_text }}</span>

    <template #icon>
      <n-flex :size="[5, 0]">
        <n-button @click="show_edit = true" type="warning" text size="small">
          <n-icon>
            <Edit />
          </n-icon>
        </n-button>
        <n-button @click="emit_delete" type="error" text size="small">
          <n-icon>
            <Delete />
          </n-icon>
        </n-button>
      </n-flex>
    </template>

    <SourceBindingModal
      @commit="emit_commit"
      v-model:show="show_edit"
      :source="source"
      :allowed-service-kinds="allowedServiceKinds"
    />
  </n-tag>
</template>
