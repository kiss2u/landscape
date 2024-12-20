<script setup lang="ts">
import { get_cpu } from "@/api/sys";
import { ref } from "vue";
import SourceProgress from "@/components/SouceProgress.vue";
import { useSysInfo } from "@/stores/systeminfo";

let sysinfo = useSysInfo();
</script>
<template>
  <!-- {{ cpus }} -->
  <SourceProgress
    v-for="each_cpu of sysinfo.cpus"
    :value="each_cpu.usage / 100"
  >
  </SourceProgress>
</template>
