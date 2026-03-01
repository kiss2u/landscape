<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { getFlowRules } from "@landscape-router/types/api/flow-rules/flow-rules";

const { t } = useI18n();

interface Props {
  placeholder?: string;
  disabled?: boolean;
  clearable?: boolean;
  filterable?: boolean;
  includeAll?: boolean; // 是否包含"全部"选项
  width?: string;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: undefined,
  disabled: false,
  clearable: true,
  filterable: true,
  includeAll: true,
  width: "150px",
});

const flowId = defineModel<number | null>();

const flowRules = ref<any[]>([]);
const loading = ref(false);

const flowOptions = computed(() => {
  const options = flowRules.value.map((e) => ({
    value: e.flow_id,
    label: e.remark ? `${e.flow_id} - ${e.remark}` : `Flow ${e.flow_id}`,
  }));

  if (props.includeAll) {
    return [{ label: t("flow.select.all"), value: null }, ...options];
  }

  return options;
});

async function loadFlowRules() {
  loading.value = true;
  try {
    flowRules.value = await getFlowRules();
  } catch (error) {
    console.error(t("flow.select.get_list_failed"), error);
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadFlowRules();
});

// 暴露刷新方法，供外部调用
defineExpose({
  refresh: loadFlowRules,
});
</script>

<template>
  <n-select
    v-model:value="flowId"
    :placeholder="placeholder ?? t('flow.select.select_flow')"
    :options="flowOptions"
    :disabled="disabled || loading"
    :filterable="filterable"
    :clearable="clearable"
    :loading="loading"
    :style="{ width }"
  />
</template>
