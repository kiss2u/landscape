<script setup lang="ts">
import { DockerImageSummary } from "@/lib/docker";
import { computed, onMounted, ref } from "vue";

const props = defineProps<{
  image: DockerImageSummary;
}>();

const title = computed(() => {
  let t = "";
  if (props.image.RepoTags != undefined && props.image.RepoTags.length > 0) {
    t = props.image.RepoTags[0].replace(/^\/+/, "");
  }
  return t;
});

const time = computed(() => {
  let t = undefined;
  if (props.image.Created != undefined) {
    t = props.image.Created * 1000;
  }
  return t;
});

const show_create_model = ref(false);
</script>
<template>
  <n-card :title="title">
    <template #header-extra>
      <n-flex>
        <n-button @click="show_create_model = true">create </n-button>
      </n-flex>
    </template>

    <n-descriptions :column="1" label-placement="left">
      <n-descriptions-item label="创建时间">
        <n-time v-if="time !== undefined" :time="time" />
        <span v-else>N/A</span>
      </n-descriptions-item>
    </n-descriptions>

    <!-- {{ props.container }} -->

    <!-- <template #action>
      <n-flex>
        <n-button>start</n-button>
        <n-button>stop</n-button>
        <n-button>delete</n-button>
      </n-flex>
    </template> -->
    <ContainerRunModal
      :image_name="title"
      v-model:show="show_create_model"
    ></ContainerRunModal>
  </n-card>
</template>
