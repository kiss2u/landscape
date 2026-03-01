<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { HelpFilled } from "@vicons/carbon";
import type { LDIAPrefix } from "@/api/service_ipv6pd";
import { useFrontEndStore } from "@/stores/front_end_config";
import { usePreferenceStore } from "@/stores/preference";
const prefStore = usePreferenceStore();
const { t } = useI18n();

const frontEndStore = useFrontEndStore();

interface Props {
  config: LDIAPrefix | null;
  iface_name: string;
  show_action?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  show_action: false,
});

const emit = defineEmits(["refresh"]);

async function refresh() {
  emit("refresh");
}
const status = computed(() => {
  if (props.config) {
    if (
      props.config.last_update_time + props.config.valid_lifetime * 1000 >
      new Date().getTime()
    ) {
      return true;
    }
  }

  return false;
});
</script>

<template>
  <n-card
    style="min-height: 224px"
    content-style="display: flex"
    size="small"
    :hoverable="true"
  >
    <template #header>
      <StatusTitle :enable="status" :remark="props.iface_name"></StatusTitle>
    </template>
    <!-- {{ config }} -->
    <n-descriptions
      v-if="config"
      style="flex: 1"
      bordered
      label-placement="top"
      :column="3"
    >
      <n-descriptions-item>
        <template #label>
          <n-flex align="center">
            <span> {{ t("lan_ipv6.prefix_info.ip_preferred_time") }} </span>
            <n-popover trigger="hover">
              <template #trigger>
                <n-button text>
                  <template #icon>
                    <n-icon><HelpFilled /></n-icon>
                  </template>
                </n-button>
              </template>
              <span>{{
                t("lan_ipv6.prefix_info.ip_preferred_time_desc")
              }}</span>
            </n-popover>
          </n-flex>
        </template>
        {{ config.preferred_lifetime }}s
      </n-descriptions-item>
      <n-descriptions-item>
        <template #label>
          <n-flex align="center">
            <span> {{ t("lan_ipv6.prefix_info.ip_valid_time") }} </span>
            <n-popover trigger="hover">
              <template #trigger>
                <n-button text>
                  <template #icon>
                    <n-icon><HelpFilled /></n-icon>
                  </template>
                </n-button>
              </template>
              <span>{{ t("lan_ipv6.prefix_info.ip_valid_time_desc") }}</span>
            </n-popover>
          </n-flex>
        </template>
        {{ config.valid_lifetime }}s
      </n-descriptions-item>
      <n-descriptions-item :label="t('lan_ipv6.prefix_info.prefix')">
        {{ frontEndStore.MASK_INFO(config.prefix_ip) }}/{{ config.prefix_len }}
      </n-descriptions-item>
      <n-descriptions-item :span="3">
        <template #label>
          <n-flex align="center">
            <span>{{ t("lan_ipv6.prefix_info.last_update") }}</span>
            <n-popover trigger="hover">
              <template #trigger>
                <n-button text>
                  <template #icon>
                    <n-icon><HelpFilled /></n-icon>
                  </template>
                </n-button>
              </template>
              <span>{{
                t("lan_ipv6.prefix_info.dhcpv6_client_prefix_time")
              }}</span>
            </n-popover>
          </n-flex>
        </template>
        <n-time
          :time="config.last_update_time"
          format="yyyy-MM-dd hh:mm:ss"
          :time-zone="prefStore.timezone"
        />
      </n-descriptions-item>
    </n-descriptions>
    <n-flex
      align="center"
      justify="center"
      style="height: 190px; flex: 1"
      v-else
    >
      <n-empty :description="t('lan_ipv6.prefix_info.no_prefix_yet')">
      </n-empty>
    </n-flex>
  </n-card>
</template>
