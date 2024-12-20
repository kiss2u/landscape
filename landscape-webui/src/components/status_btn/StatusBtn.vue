<script setup lang="ts">
import { computed } from "vue";
import { ServiceStatus, ServiceStatusType } from "@/lib/services";

const iface_info = defineProps<{
  status: ServiceStatus | undefined;
}>();

const control_show = computed(() => {
  let info = {
    btn_type: "default",
    btn_message: "未配置",
    disabled_popover: true,
  };
  if (iface_info.status != undefined) {
    switch (iface_info.status.t) {
      case ServiceStatusType.Staring: {
        info.btn_type = "success";
        info.btn_message = "启动中";
        break;
      }
      case ServiceStatusType.Running: {
        info.btn_type = "success";
        info.btn_message = "运行中";
        break;
      }
      case ServiceStatusType.Stopping: {
        info.btn_type = "success";
        info.btn_message = "停止中";
        break;
      }
      case ServiceStatusType.Stop: {
        if (iface_info.status.message == undefined) {
          info.btn_type = "default";
        } else {
          info.btn_type = "error";
          info.disabled_popover = false;
        }
        info.btn_message = "停止";
        break;
      }
    }
  } else {
  }
  return info;
});

const emit = defineEmits(["click"]);
</script>

<template>
  <n-popover
    trigger="hover"
    :show-arrow="false"
    :disabled="control_show.disabled_popover"
  >
    <template #trigger>
      <n-button
        size="tiny"
        strong
        ghost
        @click="emit('click')"
        :focusable="false"
        :type="control_show.btn_type"
      >
        <template #icon>
          <slot name="btn-icon"> </slot>
        </template>

        {{ control_show.btn_message }}
      </n-button>
    </template>
    <n-flex vertical>
      {{ iface_info.status?.message ?? "" }}
    </n-flex>
  </n-popover>
</template>
