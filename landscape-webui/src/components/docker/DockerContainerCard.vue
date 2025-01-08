<script setup lang="ts">
import {
  remove_container,
  start_container,
  stop_container,
} from "@/api/docker_service";
import { DockerContainerSummary, DockerBtnShow } from "@/lib/docker";
import { useDockerStore } from "@/stores/status_docker";
import { computed, onMounted, ref } from "vue";

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
async function stop() {
  if (title.value) {
    await stop_container(title.value);
    await dockerStore.UPDATE_INFO();
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
  <n-card :title="title">
    <template #header-extra>
      <n-ellipsis style="max-width: 240px">
        {{ props.container.Image }}
      </n-ellipsis>
    </template>

    <n-descriptions :column="1" label-placement="left">
      <n-descriptions-item
        v-if="props.container.get_redirect_id() !== undefined"
        label="接收的转发 ID 为"
      >
        {{ props.container.get_redirect_id() }}
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
      <n-flex>
        <n-button @click="start" :disabled="!show_btn.start">start</n-button>
        <n-button @click="stop" :disabled="!show_btn.stop">stop</n-button>
        <n-button @click="remove" :disabled="!show_btn.remove">remove</n-button>
      </n-flex>
    </template>
  </n-card>
</template>
