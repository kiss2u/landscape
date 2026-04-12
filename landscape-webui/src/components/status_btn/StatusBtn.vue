<script setup lang="ts">
import { computed } from "vue";
import {
  ServiceStatus,
  get_service_status_label,
  get_service_status_tag_type,
} from "@/lib/services";
import { useI18n } from "vue-i18n";

interface Props {
  status?: ServiceStatus;
  disable_popover?: boolean;
}

const iface_info = withDefaults(defineProps<Props>(), {
  disable_popover: true,
});
const { t } = useI18n();

const popover_show = computed(() => {
  return control_show.value.disabled_popover && iface_info.disable_popover;
});
const control_show = computed(() => {
  let info = {
    btn_type: "default",
    btn_message: t("common.not_configured"),
    disabled_popover: true,
  };
  if (iface_info.status != undefined) {
    info.btn_type = get_service_status_tag_type(iface_info.status);
    info.btn_message = get_service_status_label(iface_info.status, t);
  } else {
  }
  return info;
});

const emit = defineEmits(["click", "hover", "update:show"]);
</script>

<template>
  <n-popover
    trigger="hover"
    :show-arrow="false"
    @update:show="(show: boolean) => emit('update:show', show)"
    :disabled="popover_show"
  >
    <template #trigger>
      <n-button
        size="tiny"
        strong
        ghost
        @click="emit('click')"
        :focusable="false"
        :type="control_show.btn_type"
        style="min-width: 67px"
      >
        <template #icon>
          <slot name="btn-icon"> </slot>
        </template>

        {{ control_show.btn_message }}
      </n-button>
    </template>
    <n-flex vertical>
      <slot name="popover-panel">
        <!-- {{ iface_info.status?.message ?? "" }} -->
      </slot>
    </n-flex>
  </n-popover>
</template>
