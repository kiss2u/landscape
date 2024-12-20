<script setup lang="ts">
import { get_mem } from "@/api/sys";
import { computed, ref } from "vue";
import SourceProgress from "@/components/SouceProgress.vue";

import { useSysInfo } from "@/stores/systeminfo";

let sysinfo = useSysInfo();

const percentage = computed(() => {
  // console.log(sysinfo.mem.used_mem / sysinfo.mem.total_mem);
  return sysinfo.mem.used_mem / sysinfo.mem.total_mem;
});
</script>
<template>
  <!-- {{ mem }} -->
  <SourceProgress :value="percentage"></SourceProgress>
</template>
