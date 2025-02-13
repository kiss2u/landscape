<script setup lang="ts">
import { ref } from "vue";

const ip = defineModel<number[]>("ip", { required: true });
const mask = defineModel<number | undefined>("mask");
</script>

<template>
  <n-input-group>
    <n-input-number
      v-model:value="ip[0]"
      :show-button="false"
      min="0"
      max="255"
      placeholder=""
    />
    <n-input-group-label>.</n-input-group-label>
    <n-input-number
      v-model:value="ip[1]"
      :show-button="false"
      min="0"
      max="255"
      placeholder=""
    />
    <n-input-group-label>.</n-input-group-label>
    <n-input-number
      v-model:value="ip[2]"
      :show-button="false"
      min="0"
      max="255"
      placeholder=""
    />
    <n-input-group-label>.</n-input-group-label>
    <n-input-number
      v-model:value="ip[3]"
      :show-button="false"
      min="0"
      max="254"
      placeholder=""
    />
    <n-input-group-label v-if="mask !== undefined">/</n-input-group-label>
    <n-input-number
      v-if="mask !== undefined"
      v-model:value="mask"
      placeholder="mask"
    />
  </n-input-group>
</template>
