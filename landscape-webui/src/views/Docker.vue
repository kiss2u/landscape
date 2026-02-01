<script setup lang="ts">
import DockerAllContainer from "@/components/docker/DockerAllContainer.vue";
import DockerStatusCard from "@/components/docker/DockerStatusCard.vue";
</script>
<template>
  <n-layout :native-scrollbar="false" content-style="padding: 10px;">
    <n-flex style="flex: 1; padding-right: 15px" vertical>
      <n-flex><DockerStatusCard></DockerStatusCard> </n-flex>
      <n-flex style="flex: 1">
        <DockerAllContainer></DockerAllContainer>
      </n-flex>
    </n-flex>
  </n-layout>
</template>
