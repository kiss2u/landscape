<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();

const goHome = () => {
  router.push("/");
};
</script>

<template>
  <div class="not-found">
    <n-result
      status="404"
      title="404 资源不存在"
      description="您访问的路径可能已被移除，或从未存在过。"
    >
      <template #footer>
        <n-button type="primary" secondary @click="goHome"> 返回首页 </n-button>
      </template>
    </n-result>
  </div>
</template>

<style scoped>
.not-found {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
}
</style>
