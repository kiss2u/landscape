<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();

const goBack = () => {
  const target = (history.state?.redirect as string) || "/";
  router.push(target);
};
</script>

<template>
  <div class="unavailable">
    <n-result
      status="warning"
      :title="$t('unavailable.unavailable_title')"
      :description="$t('unavailable.unavailable_desc')"
    >
      <template #footer>
        <n-button type="primary" secondary @click="goBack">
          {{ $t("unavailable.go_back") }}
        </n-button>
      </template>
    </n-result>
  </div>
</template>

<style scoped>
.unavailable {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
}
</style>
