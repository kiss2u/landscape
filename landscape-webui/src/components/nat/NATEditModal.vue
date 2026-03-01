<script setup lang="ts">
import { ZoneType, IfaceIpMode } from "@/lib/service_ipconfig";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import Range from "@/components/PortRange.vue";
import { NatServiceConfig } from "@/lib/nat";
import {
  get_iface_nat_config,
  update_iface_nat_config,
} from "@/api/service_nat";
import { useNATConfigStore } from "@/stores/status_nats";
import { IfaceZoneType } from "@landscape-router/types/api/schemas";

let natConfigStore = useNATConfigStore();
const { t } = useI18n();
const show_model = defineModel<boolean>("show", { required: true });
const emit = defineEmits(["refresh"]);

const iface_info = defineProps<{
  iface_name: string;
  zone: IfaceZoneType;
}>();

const nat_service_config = ref<NatServiceConfig>(
  new NatServiceConfig({
    iface_name: iface_info.iface_name,
  }),
);

async function on_modal_enter() {
  try {
    let config = await get_iface_nat_config(iface_info.iface_name);
    console.log(config);
    // iface_service_type.value = config.t;
    nat_service_config.value = config;
  } catch (e) {
    nat_service_config.value = new NatServiceConfig({
      iface_name: iface_info.iface_name,
    });
  }
}

async function save_config() {
  let config = await update_iface_nat_config(nat_service_config.value);
  await natConfigStore.UPDATE_INFO();
  show_model.value = false;
}
</script>

<template>
  <n-modal
    :auto-focus="false"
    v-model:show="show_model"
    @after-enter="on_modal_enter"
  >
    <n-card
      style="width: 600px"
      :title="t('nat.service_edit.title')"
      :bordered="false"
      size="small"
      role="dialog"
      aria-modal="true"
    >
      <n-form :model="nat_service_config">
        <n-form-item :label="t('common.enable_question')">
          <n-switch v-model:value="nat_service_config.enable">
            <template #checked> {{ t("common.enable") }} </template>
            <template #unchecked> {{ t("common.disable") }} </template>
          </n-switch>
        </n-form-item>
        <n-form-item :label="t('nat.service_edit.tcp_port_range')">
          <Range v-model:range="nat_service_config.nat_config.tcp_range">
          </Range>
        </n-form-item>
        <n-form-item :label="t('nat.service_edit.udp_port_range')">
          <Range v-model:range="nat_service_config.nat_config.udp_range">
          </Range>
        </n-form-item>
        <n-form-item :label="t('nat.service_edit.icmp_id_range')">
          <Range v-model:range="nat_service_config.nat_config.icmp_in_range">
          </Range>
        </n-form-item>
      </n-form>

      <template #footer>
        <n-flex justify="end">
          <n-button round type="primary" @click="save_config">
            {{ t("common.update") }}
          </n-button>
        </n-flex>
      </template>
    </n-card>
  </n-modal>
</template>
