<script lang="ts" setup>
import { useThemeVars } from "naive-ui";
import { DotMark } from "@vicons/carbon";
import { ref } from "vue";

const themeVars = ref(useThemeVars());
interface Props {
  ip_status?: boolean[];
}

withDefaults(defineProps<Props>(), { ip_status: () => [] });
</script>
<template>
  <n-flex justify="center" align="center" style="width: 168px" :size="[0, 0]">
    <n-icon
      v-for="enable in ip_status"
      :color="enable ? themeVars.successColor : ''"
      size="14"
    >
      <DotMark />
    </n-icon>
  </n-flex>
</template>
