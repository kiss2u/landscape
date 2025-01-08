<script setup lang="ts">
import { computed, ref } from "vue";

import { ServiceStatusType } from "@/lib/services";
import { useDockerStore } from "@/stores/status_docker";

import DockerImageDrawer from "@/components/docker/DockerImageDrawer.vue";
import {
  start_docker_service,
  stop_docker_service,
} from "@/api/docker_service";

const dockerStatus = useDockerStore();

const show_image_drawer = ref(false);

const is_down = computed(() => {
  return dockerStatus.docker_status.t == ServiceStatusType.Stop;
});

async function start() {
  await start_docker_service();
}
async function stop() {
  await stop_docker_service();
}
</script>
<template>
  <n-card title="Docker">
    <template #header-extra>
      <n-flex>
        <n-button @click="show_image_drawer = true">查看镜像</n-button>
        <n-button @click="start" v-if="is_down"> 开启 </n-button>
        <n-button v-else @click="stop">关闭 docker 事件监听服务</n-button>
      </n-flex>
    </template>
    // TODO 展示使用资源
    {{ dockerStatus.docker_status }}
    <!-- <template #footer> #footer </template>
    <template #action> #action </template> -->
    <DockerImageDrawer v-model:show="show_image_drawer" />
  </n-card>
</template>
