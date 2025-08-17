<script setup lang="ts">
import { get_flow_rules } from "@/api/flow";
import { FlowMarkType } from "@/lib/default_value";
import { FlowMark } from "@/rust_bindings/flow";
import { computed, onMounted, ref } from "vue";

const mark = defineModel<FlowMark>("mark", { required: true });

const mark_type_option = [
  {
    label: "无动作",
    value: FlowMarkType.KeepGoing,
  },
  {
    label: "忽略 Flow 设置",
    value: FlowMarkType.Direct,
  },
  {
    label: "禁止连接",
    value: FlowMarkType.Drop,
  },
  {
    label: "重定向至流",
    value: FlowMarkType.Redirect,
  },
  // {
  //   label: "允许端口共享",
  //   value: FlowMarkType.AllowReusePort,
  // },
];

onMounted(async () => {
  await search_flows();
});

const flow_rules = ref<any[]>([]);
const flow_options = computed(() => {
  return flow_rules.value.map((e) => ({
    value: e.flow_id,
    label: e.remark ? `${e.flow_id} - ${e.remark}` : e.flow_id,
  }));
});
const flow_search_loading = ref(false);
async function search_flows() {
  flow_rules.value = await get_flow_rules();
}
</script>

<template>
  <n-input-group>
    <n-select
      style="width: 50%"
      v-model:value="mark.t"
      :options="mark_type_option"
      placeholder="选择匹配方式"
    />
    <n-select
      style="width: 50%"
      v-if="mark.t === FlowMarkType.Redirect"
      v-model:value="mark.flow_id"
      filterable
      placeholder="重定向的流 ID"
      :options="flow_options"
      :loading="flow_search_loading"
      clearable
      remote
      @search="search_flows"
    />
  </n-input-group>
</template>
