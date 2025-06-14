<script setup lang="ts">
import { update_iface_pppd_config } from "@/api/service_pppd";
import { PPPDConfig, PPPDServiceConfig } from "@/lib/pppd";
import { computed, ref } from "vue";
import { useFrontEndStore } from "@/stores/front_end_config";

const frontEndStore = useFrontEndStore();
const show = defineModel<boolean>("show", { required: true });
const props = defineProps<{
  attach_iface_name: string;
  origin_value: PPPDServiceConfig | undefined;
}>();

// const origin_value = defineModel<PPPDServiceConfig>("config", {
//   required: true,
// });

const emit = defineEmits(["refresh"]);
const value = ref<PPPDServiceConfig>(
  new PPPDServiceConfig({
    attach_iface_name: props.attach_iface_name,
  })
);

const isModified = computed(() => {
  return JSON.stringify(value.value) !== JSON.stringify(props.origin_value);
});

async function init_conf_value() {
  value.value = new PPPDServiceConfig(
    props.origin_value
      ? props.origin_value
      : {
          attach_iface_name: props.attach_iface_name,
        }
  );
}

async function confirm_config() {
  if (isModified) {
    await update_iface_pppd_config(value.value);
    show.value = false;
    emit("refresh");
  }
}
</script>
<template>
  <n-modal
    v-model:show="show"
    preset="card"
    style="width: 600px"
    title="编辑 PPPD 服务"
    @after-enter="init_conf_value"
  >
    <!-- <template #header-extra> 噢! </template> -->
    <!-- {{ origin_value }} -->

    <n-form style="flex: 1" ref="formRef" :model="value" :cols="4">
      <n-grid :cols="5">
        <n-form-item-gi label="启用" :span="1">
          <n-switch v-model:value="value.enable">
            <template #checked> 启用 </template>
            <template #unchecked> 禁用 </template>
          </n-switch>
        </n-form-item-gi>

        <n-form-item-gi :span="2" label="设置默认路由">
          <n-switch v-model:value="value.pppd_config.default_route">
            <template #checked> 启用 </template>
            <template #unchecked> 禁用 </template>
          </n-switch>
        </n-form-item-gi>

        <n-form-item-gi label="ppp网口名称" :span="2">
          <n-input v-model:value="value.iface_name" clearable />
        </n-form-item-gi>
      </n-grid>

      <n-form-item label="用户名">
        <n-input
          :type="frontEndStore.presentation_mode ? 'password' : 'text'"
          show-password-on="click"
          v-model:value="value.pppd_config.peer_id"
        />
      </n-form-item>

      <n-form-item label="密码">
        <n-input
          :type="frontEndStore.presentation_mode ? 'password' : 'text'"
          show-password-on="click"
          v-model:value="value.pppd_config.password"
        />
      </n-form-item>
    </n-form>
    <template #footer>
      <n-flex justify="space-between">
        <n-button @click="show = false">取消</n-button>
        <n-button
          @click="confirm_config()"
          type="success"
          :disabled="!isModified"
        >
          确定
        </n-button>
      </n-flex>
    </template>
  </n-modal>
</template>
