<script setup lang="ts">
import { FlowEntryRule } from "landscape-types/common/flow";
import { useFrontEndStore } from "@/stores/front_end_config";

interface Prop {
  rule: FlowEntryRule;
}

const frontEndStore = useFrontEndStore();
defineProps<Prop>();
</script>

<template>
  <n-tag :bordered="false" v-if="rule.mode.t === 'mac'">
    {{ frontEndStore.MASK_INFO(rule.mode.mac_addr) }}
  </n-tag>
  <n-tag :bordered="false" v-else>
    {{ `${frontEndStore.MASK_INFO(rule.mode.ip)}/${rule.mode.prefix_len}` }}
  </n-tag>
</template>
