<script setup lang="ts">
import { computed, ref } from "vue";

import {
  remove_container,
  start_container,
  stop_container,
} from "@/api/docker_service";
import { DockerContainerSummary, DockerBtnShow } from "@/lib/docker";
import { useDockerStore } from "@/stores/status_docker";

const props = defineProps<{
  container: DockerContainerSummary;
}>();

const dockerStore = useDockerStore();

const title = computed(() => {
  let t = undefined;
  if (props.container.Names != undefined && props.container.Names.length > 0) {
    t = props.container.Names[0].replace(/^\/+/, "");
  }
  return t;
});

const time = computed(() => {
  let t = undefined;
  if (props.container.Created != undefined) {
    t = props.container.Created * 1000;
  }
  return t;
});

const show_btn = computed(() => new DockerBtnShow(props.container.State));
async function start() {
  if (title.value) {
    await start_container(title.value);
    await dockerStore.UPDATE_INFO();
  }
}

const stop_spin = ref(false);
const show_stop_popconfirm = ref(false);
async function stop() {
  show_stop_popconfirm.value = false;
  if (title.value) {
    try {
      stop_spin.value = true;
      await stop_container(title.value);
    } catch (e) {
    } finally {
      stop_spin.value = false;
      await dockerStore.UPDATE_INFO();
    }
  }
}

async function remove() {
  if (title.value) {
    await remove_container(title.value);
    await dockerStore.UPDATE_INFO();
  }
}
</script>
<template>
  <n-card class="docker-container-exhibit-card" size="small">
    <template #header>
      <!-- <n-marquee :speed="13">
        {{ title }}
      </n-marquee> -->
      <n-ellipsis>
        {{ title }}
      </n-ellipsis>
    </template>
    <template #header-extra>
      <n-flex>
        <n-button
          secondary
          size="small"
          @click="start"
          type="success"
          :disabled="!show_btn.start"
        >
          start
        </n-button>
        <n-popconfirm
          v-model:show="show_stop_popconfirm"
          @positive-click="stop"
        >
          <template #trigger>
            <n-button
              :loading="stop_spin"
              secondary
              size="small"
              @click="show_stop_popconfirm = true"
              type="warning"
              :disabled="!show_btn.stop"
            >
              stop
            </n-button>
          </template>
          确定停止吗
        </n-popconfirm>

        <n-popconfirm @positive-click="remove">
          <template #trigger>
            <n-button
              secondary
              size="small"
              type="error"
              :disabled="!show_btn.remove"
            >
              remove
            </n-button>
          </template>
          确定删除吗
        </n-popconfirm>
      </n-flex>
    </template>

    <n-descriptions :column="1" label-placement="left">
      <n-descriptions-item label="镜像">
        <n-ellipsis style="max-width: 220px">
          {{ props.container.Image }}
        </n-ellipsis>
      </n-descriptions-item>
      <n-descriptions-item label="状态">
        {{ props.container.State }}
      </n-descriptions-item>

      <n-descriptions-item label="创建时间">
        <n-time v-if="time !== undefined" :time="time" />
        <span v-else>N/A</span>
      </n-descriptions-item>
    </n-descriptions>

    <!-- {{ props.container }} -->

    <template #action>
      <n-tag v-for="tag of props.container.Labels" :bordered="false">
        {{ `${tag[0]}: ${tag[1]}` }}
      </n-tag>
    </template>
  </n-card>
</template>
<style scoped>
.docker-container-exhibit-card {
  flex: 1;
}
</style>
