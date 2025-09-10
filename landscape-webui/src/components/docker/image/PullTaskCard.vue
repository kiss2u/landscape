<script setup lang="ts">
import { get_current_tasks } from "@/api/docker";
import { PullManagerInfo } from "@/rust_bindings/common/docker";
import useDockerImgTask from "@/stores/docker_img_task";
import { onMounted, ref } from "vue";

const dockerImgTask = useDockerImgTask();
const tasks = ref<PullManagerInfo>();

onMounted(async () => {
  await read_task();
});

async function read_task() {
  tasks.value = await get_current_tasks();
}
</script>

<template>
  <n-flex>
    {{ tasks }}
  </n-flex>
</template>
