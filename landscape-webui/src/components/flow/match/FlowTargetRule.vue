<script setup lang="ts">
import { FlowTarget } from "@/rust_bindings/common/flow";
import { useFrontEndStore } from "@/stores/front_end_config";

const frontEndStore = useFrontEndStore();
const target_rules = defineModel<FlowTarget[]>("target_rules", {
  required: true,
});

enum FlowTargetEnum {
  Interface = "interface",
  NetNS = "netns",
}

function onCreate(): FlowTarget {
  return { t: "interface", name: "" };
}

function target_type_option(): any[] {
  return [
    {
      label: "网卡",
      value: "interface",
    },
    {
      label: "docker",
      value: "netns",
    },
  ];
}

function handleUpdateValue(value: FlowTarget, index: number) {
  if (value.t == FlowTargetEnum.Interface) {
    target_rules.value[index] = {
      t: FlowTargetEnum.Interface,
      name: "",
    };
  } else {
    target_rules.value[index] = { t: FlowTargetEnum.NetNS, container_name: "" };
  }
}
</script>

<template>
  <n-dynamic-input
    :min="0"
    :max="1"
    v-model:value="target_rules"
    :on-create="onCreate"
  >
    <template #create-button-default> 增加一条出口规则 </template>
    <template #default="{ value, index }">
      <n-input-group>
        <n-select
          :style="{ width: '33%' }"
          v-model:value="value.t"
          @update:value="handleUpdateValue(value, index)"
          :options="target_type_option()"
        />

        <n-input
          v-if="value.t == 'interface'"
          :type="frontEndStore.presentation_mode ? 'password' : 'text'"
          v-model:value="value.name"
          :style="{ width: '66%' }"
          placeholder="网卡名称"
        />
        <n-input
          v-else-if="value.t == 'netns'"
          :type="frontEndStore.presentation_mode ? 'password' : 'text'"
          v-model:value="value.container_name"
          :style="{ width: '66%' }"
          placeholder="容器名称"
        />
      </n-input-group>
    </template>
  </n-dynamic-input>
</template>
