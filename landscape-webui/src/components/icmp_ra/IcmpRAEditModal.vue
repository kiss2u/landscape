<script setup lang="ts">
import { computed, onActivated, onMounted, ref, watch } from "vue";
import { FormInst, useMessage } from "naive-ui";
import { ZoneType } from "@/lib/service_ipconfig";
import { useIPv6PDStore } from "@/stores/status_ipv6pd";
import { IPV6RAServiceConfig } from "@/lib/icmpv6ra";
import {
  get_iface_icmpv6ra_config,
  update_icmpv6ra_config,
} from "@/api/service_icmpv6ra";
import { get_all_ipv6pd_status } from "@/api/service_ipv6pd";
import { ServiceStatus } from "@/lib/services";

let ipv6PDStore = useIPv6PDStore();
const message = useMessage();

const show_model = defineModel<boolean>("show", { required: true });
const emit = defineEmits(["refresh"]);
const formRef = ref<FormInst | null>(null);

const iface_info = defineProps<{
  iface_name: string;
  mac?: string;
  zone: ZoneType;
}>();

const service_config = ref<IPV6RAServiceConfig>(
  new IPV6RAServiceConfig({
    iface_name: iface_info.iface_name,
  })
);

async function on_modal_enter() {
  try {
    let config = await get_iface_icmpv6ra_config(iface_info.iface_name);
    console.log(config);
    // iface_service_type.value = config.t;
    service_config.value = config;
  } catch (e) {
    new IPV6RAServiceConfig({
      iface_name: iface_info.iface_name,
    });
  } finally {
    await search_ipv6_pd();
  }
}

async function save_config() {
  try {
    await formRef.value?.validate();
    let config = await update_icmpv6ra_config(service_config.value);
    await ipv6PDStore.UPDATE_INFO();
    show_model.value = false;
  } catch (err) {
    message.warning(`表单校验未通过`);
  }
}

const ipv6_pd_ifaces = ref<Map<string, ServiceStatus>>(new Map());
const loading_search_ipv6pd = ref(false);

const ipv6_pd_options = computed(() => {
  const result = [];
  for (const [key, value] of ipv6_pd_ifaces.value) {
    result.push({ value: key, label: `${key} - ${value.t}` });
  }
  return result;
});

async function search_ipv6_pd() {
  ipv6_pd_ifaces.value = await get_all_ipv6pd_status();
}

const formRules = {
  config: {
    depend_iface: {
      required: true,
      message: "请选择用于申请前缀的网卡",
      trigger: ["blur", "change"],
    },
  },
};
</script>

<template>
  <n-modal
    :auto-focus="false"
    v-model:show="show_model"
    @after-enter="on_modal_enter"
  >
    <n-card
      style="width: 600px"
      title="ICMPv6 RA 配置"
      :bordered="false"
      size="small"
      role="dialog"
      aria-modal="true"
      closable
      @close="show_model = false"
    >
      <!-- {{ service_config }} -->
      <n-form ref="formRef" :model="service_config" :rules="formRules">
        <n-grid :x-gap="12" :y-gap="8" cols="4" item-responsive>
          <n-form-item-gi span="4 m:4 l:4" label="是否启用">
            <n-switch v-model:value="service_config.enable">
              <template #checked> 启用 </template>
              <template #unchecked> 禁用 </template>
            </n-switch>
          </n-form-item-gi>

          <n-form-item-gi
            span="4 m:4 l:4"
            label="所关联的网卡 (须对应网卡开启 DHCPv6-PD)"
            path="config.depend_iface"
          >
            <n-select
              v-model:value="service_config.config.depend_iface"
              filterable
              placeholder="选择进行前缀申请的网卡"
              :options="ipv6_pd_options"
              :loading="loading_search_ipv6pd"
              clearable
              remote
              @search="search_ipv6_pd"
            />

            <!-- <n-input
              style="flex: 1"
              v-model:value="service_config.config.depend_iface"
              clearable
            /> -->
          </n-form-item-gi>

          <n-form-item-gi span="2 m:2 l:2" label="子网索引">
            <n-input-number
              style="flex: 1"
              v-model:value="service_config.config.subnet_index"
              clearable
            />
          </n-form-item-gi>
          <n-form-item-gi span="2 m:2 l:2" label="子网前缀长度">
            <n-input-number
              style="flex: 1"
              v-model:value="service_config.config.subnet_prefix"
              clearable
            />
          </n-form-item-gi>

          <n-form-item-gi span="2 m:2 l:2" label="IP 首选状态时间 (s)">
            <n-input-number
              style="flex: 1"
              v-model:value="service_config.config.ra_preferred_lifetime"
              clearable
            />
          </n-form-item-gi>
          <n-form-item-gi span="2 m:2 l:2" label="IP 有效状态时间 (s)">
            <n-input-number
              style="flex: 1"
              v-model:value="service_config.config.ra_valid_lifetime"
              clearable
            />
          </n-form-item-gi>

          <!-- flag 部分 -->
          <n-form-item-gi span="2 m:2" label="使用 DHCPv6 获取 IPv6 地址">
            <n-switch
              v-model:value="
                service_config.config.ra_flag.managed_address_config
              "
            />
          </n-form-item-gi>
          <n-form-item-gi span="2 m:2" label="使用 DHCPv6 获取 其他信息">
            <n-switch
              v-model:value="service_config.config.ra_flag.other_config"
            />
          </n-form-item-gi>
          <n-form-item-gi span="2 m:2" label="移动 IPv6 归属代理">
            <n-switch
              v-model:value="service_config.config.ra_flag.home_agent"
            />
          </n-form-item-gi>

          <n-form-item-gi span="2 m:2" label="邻居发现代理">
            <n-switch v-model:value="service_config.config.ra_flag.nd_proxy" />
          </n-form-item-gi>
          <n-form-item-gi span="4 m:4" label="默认路由优先级">
            <n-radio-group
              v-model:value="service_config.config.ra_flag.prf"
              name="ra_flag"
            >
              <n-radio-button :value="3" label="低" />
              <n-radio-button :value="0" label="中 (默认)" />
              <n-radio-button :value="1" label="高" />
            </n-radio-group>
          </n-form-item-gi>
        </n-grid>
      </n-form>
      <template #footer>
        <n-flex justify="end">
          <n-button round type="primary" @click="save_config"> 更新 </n-button>
        </n-flex>
      </template>
    </n-card>
  </n-modal>
</template>
