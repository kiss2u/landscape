<script setup lang="ts">
import { onMounted, ref } from "vue";
import { get_firewall_rules } from "@/api/mark";

const rules = ref<any>([]);

async function read_rules() {
  rules.value = await get_firewall_rules();
}

onMounted(async () => {
  await read_rules();
});
</script>
<template>
  <n-flex style="flex: 1; padding-right: 15px" vertical>
    {{ rules }}
  </n-flex>
</template>
