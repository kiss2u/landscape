<script setup lang="ts" >
import { NProgress } from "naive-ui";
import { useThemeVars } from 'naive-ui'
// import {get_cpu} from "./api/sys";
import { changeColor } from 'seemly'
import { computed, onMounted, ref } from "vue";

const props = defineProps<{
    value: number
}>()

const themeVars = ref(useThemeVars());
const percentage = computed(() => {
  return parseFloat((props.value * 100).toFixed(1));
});

const percentage_color = computed(() => {
    if (percentage.value > 90) {
        return themeVars.value.errorColor;
    } else if (percentage.value > 80) {
        return themeVars.value.warningColor;
    } else {
        return themeVars.value.successColor;
    }

    // return themeVars.value.successColor;
})

</script>
<template>
    <!-- {{ themeVars }} -->
    <n-progress
    type="dashboard"
    gap-position="bottom"
    :color="percentage_color"
    :rail-color="changeColor(percentage_color, { alpha: 0.2 })"
    :percentage="percentage"
  />
</template>