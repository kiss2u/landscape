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
      :title="$t('error.not_found_title')"
      :description="$t('error.not_found_desc')"
    >
      <template #footer>
        <n-button type="primary" secondary @click="goHome">
          {{ $t("error.back_home") }}
        </n-button>
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
