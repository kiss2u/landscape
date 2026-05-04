<script setup lang="ts">
import { computed, ref } from "vue";
import { ZoneType, IfaceIpMode } from "@/lib/service_ipconfig";
import ConfigModal from "@/components/common/ConfigModal.vue";
import { FirewallServiceConfig } from "@/lib/firewall";
import { useFirewallConfigStore } from "@/stores/status_firewall";
import {
  get_iface_firewall_config,
  update_firewall_config,
} from "@/api/service_firewall";
import { IfaceZoneType } from "@landscape-router/types/api/schemas";
import { useI18n } from "vue-i18n";

const firewallConfigStore = useFirewallConfigStore();
const { t } = useI18n();
const show_model = defineModel<boolean>("show", { required: true });
const emit = defineEmits(["refresh"]);

const iface_info = defineProps<{
  iface_name: string;
  zone: IfaceZoneType;
}>();

const service_config = ref<FirewallServiceConfig>(
  new FirewallServiceConfig({
    iface_name: iface_info.iface_name,
  }),
);

async function on_modal_enter() {
  try {
    let config = await get_iface_firewall_config(iface_info.iface_name);
    console.log(config);
    // iface_service_type.value = config.t;
    service_config.value = config;
  } catch (e) {
    service_config.value = new FirewallServiceConfig({
      iface_name: iface_info.iface_name,
    });
  }
}

async function save_config() {
  let config = await update_firewall_config(service_config.value);
  await firewallConfigStore.UPDATE_INFO();
  show_model.value = false;
}
</script>

<template>
  <ConfigModal
    v-model:show="show_model"
    v-model:enabled="service_config.enable"
    :title="t('firewall.service_edit.title')"
    width="600px"
    @after-enter="on_modal_enter"
  >
    <template #footer>
      <n-flex justify="end">
        <n-button round type="primary" @click="save_config">
          {{ t("common.update") }}
        </n-button>
      </n-flex>
    </template>
  </ConfigModal>
</template>
