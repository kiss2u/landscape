<script setup lang="ts">
import { onMounted, ref } from "vue";
import { FlowConfig } from "@/rust_bindings/flow";
import { get_flow_rules } from "@/api/flow";
import FlowEditModal from "@/components/flow/FlowEditModal.vue";

const flows = ref<FlowConfig[]>([]);

const show_edit = ref(false);
onMounted(async () => {
  flows.value = await get_flow_rules();
});
</script>
<template>
  <n-layout :native-scrollbar="false" content-style="padding: 10px;">
    <n-flex vertical>
      <n-flex> cccc</n-flex>
      <n-grid x-gap="12" y-gap="10" cols="1 600:2 900:3 1200:4 1600:5">
        <n-grid-item style="display: flex">
          <n-card size="small" content-style="display:flex;">
            <n-flex style="flex: 1" justify="center" align="center">
              <n-button @click="show_edit = true" text style="font-size: 24px">
                <n-icon size="200">
                  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
                    <path
                      d="M368.5 240H272v-96.5c0-8.8-7.2-16-16-16s-16 7.2-16 16V240h-96.5c-8.8 0-16 7.2-16 16 0 4.4 1.8 8.4 4.7 11.3 2.9 2.9 6.9 4.7 11.3 4.7H240v96.5c0 4.4 1.8 8.4 4.7 11.3 2.9 2.9 6.9 4.7 11.3 4.7 8.8 0 16-7.2 16-16V272h96.5c8.8 0 16-7.2 16-16s-7.2-16-16-16z"
                    />
                  </svg>
                </n-icon>
              </n-button>
            </n-flex>
          </n-card>
        </n-grid-item>
        <n-grid-item
          v-for="flow in flows"
          :key="flow.flow_id"
          style="display: flex"
        >
          <FlowConfigCard :config="flow"></FlowConfigCard>
        </n-grid-item>
      </n-grid>
    </n-flex>
    <FlowEditModal v-model:show="show_edit" />
  </n-layout>
</template>
