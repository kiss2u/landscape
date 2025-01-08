<script setup lang="ts">
import { get_docker_images } from "@/api/docker_service";
import { ref } from "vue";
import DockerImageCard from "@/components/docker/DockerImageCard.vue";
const show = defineModel<boolean>("show", { required: true });
const images = ref<any>([]);

async function flush_images() {
  images.value = await get_docker_images();
}

const show_create_modal = ref(false);
</script>
<template>
  <n-drawer
    @after-enter="flush_images"
    v-model:show="show"
    :width="502"
    placement="right"
  >
    <n-drawer-content title="Docker 镜像列表">
      <n-button @click="show_create_modal = true">增加规则</n-button>
      <!-- {{ images }} -->
      <n-flex vertical>
        <DockerImageCard
          v-for="image in images"
          :key="image.index"
          :image="image"
        >
        </DockerImageCard>
      </n-flex>
    </n-drawer-content>
  </n-drawer>
</template>
