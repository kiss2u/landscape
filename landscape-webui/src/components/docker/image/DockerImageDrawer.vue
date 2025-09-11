<script setup lang="ts">
import { get_docker_images, pull_docker_image } from "@/api/docker";
import { ref } from "vue";
import DockerImageCard from "@/components/docker/image/DockerImageCard.vue";
import { useMessage } from "naive-ui";
const show = defineModel<boolean>("show", { required: true });
const images = ref<any>([]);
const message = useMessage();

async function flush_images() {
  images.value = await get_docker_images();
}

async function pull_image() {
  if (
    pull_docker_image_name.value == null ||
    pull_docker_image_name.value == undefined ||
    pull_docker_image_name.value == ""
  ) {
    message.warning("拉取的镜像名称不能为空");
    return;
  }
  await pull_docker_image(pull_docker_image_name.value);
  pull_docker_image_name.value = "";
}

const pull_docker_image_name = ref("");
const show_pull_history = ref(false);
</script>
<template>
  <n-drawer
    @after-enter="flush_images()"
    v-model:show="show"
    width="500px"
    placement="right"
    responsive
  >
    <n-drawer-content title="Docker 镜像列表" closable>
      <n-flex style="height: 100%" vertical>
        <n-input-group>
          <n-input v-model:value="pull_docker_image_name" />
          <n-button @click="pull_image">拉取镜像</n-button>
          <n-button @click="show_pull_history = true">历史任务</n-button>
        </n-input-group>
        <!-- <n-input-group>
          <n-input-group-label>filter</n-input-group-label>
          <n-input />
        </n-input-group> -->
        <n-scrollbar>
          <n-flex>
            <DockerImageCard
              v-for="image in images"
              :key="image.index"
              :image="image"
              @refresh="flush_images()"
            >
            </DockerImageCard>
          </n-flex>
        </n-scrollbar>
      </n-flex>
      <ImgPullHistory
        @refresh="flush_images()"
        v-model:show="show_pull_history"
      ></ImgPullHistory>
    </n-drawer-content>
  </n-drawer>
</template>
