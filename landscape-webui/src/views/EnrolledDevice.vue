<script lang="ts" setup>
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import EnrolledDeviceCard from "@/components/device/EnrolledDeviceCard.vue";
import EnrolledDeviceEditModal from "@/components/device/EnrolledDeviceEditModal.vue";
import { Add } from "@vicons/carbon";
import { useEnrolledDeviceStore } from "@/stores/enrolled_device";

const { t } = useI18n();
const enrolledDeviceStore = useEnrolledDeviceStore();

onMounted(async () => {
  await enrolledDeviceStore.UPDATE_INFO();
});

const show_edit_modal = ref(false);
</script>

<template>
  <n-flex vertical style="flex: 1; padding: 24px">
    <n-flex align="center">
      <n-button type="primary" @click="show_edit_modal = true">
        <template #icon>
          <n-icon><Add /></n-icon>
        </template>
        {{ t("enrolled_device.add_btn") }}
      </n-button>
    </n-flex>

    <n-divider />

    <n-spin :show="enrolledDeviceStore.loading">
      <n-grid x-gap="12" y-gap="12" cols="1 600:2 1000:3 1400:4">
        <n-grid-item
          v-for="item in enrolledDeviceStore.bindings"
          :key="item.id"
        >
          <EnrolledDeviceCard :rule="item" />
        </n-grid-item>
      </n-grid>

      <n-empty
        v-if="
          enrolledDeviceStore.bindings?.length === 0 &&
          !enrolledDeviceStore.loading
        "
        :description="t('enrolled_device.empty_desc')"
        style="margin-top: 100px"
      >
        <template #extra>
          <n-button @click="show_edit_modal = true">{{
            t("enrolled_device.add_now")
          }}</n-button>
        </template>
      </n-empty>
    </n-spin>

    <EnrolledDeviceEditModal :rule_id="null" v-model:show="show_edit_modal" />
  </n-flex>
</template>

<style scoped>
.n-h2 {
  font-weight: 600;
}
</style>
