<script setup lang="ts">
import { computed } from "vue";
import { Netmask } from "netmask";

import { DhcpServerConfig, get_dhcp_range } from "@/lib/dhcp";
import NewIpEdit from "../NewIpEdit.vue";

const ip_model = defineModel<DhcpServerConfig>("config", { required: true });
const server_ip_addr = computed({
  get() {
    return ip_model.value.server_ip_addr;
  },
  set(new_value) {
    ip_model.value.server_ip_addr = new_value;
    const [start, end] = get_dhcp_range(
      `${ip_model.value.server_ip_addr}/${ip_model.value.network_mask}`
    );
    ip_model.value.ip_range_start = start;
    ip_model.value.ip_range_end = end;
  },
});

const network_mask = computed({
  get() {
    return ip_model.value.network_mask;
  },
  set(new_value) {
    ip_model.value.network_mask = new_value;
    const [start, end] = get_dhcp_range(
      `${ip_model.value.server_ip_addr}/${ip_model.value.network_mask}`
    );
    ip_model.value.ip_range_start = start;
    ip_model.value.ip_range_end = end;
  },
});
</script>
<template>
  <n-form style="flex: 1" ref="formRef" :model="ip_model" :cols="5">
    <n-grid :cols="5">
      <n-form-item-gi label="DHCP 服务 IP" :span="5">
        <NewIpEdit
          v-model:ip="server_ip_addr"
          v-model:mask="network_mask"
        ></NewIpEdit>
      </n-form-item-gi>
      <!-- <n-form-item-gi label="IP 掩码" :span="5">
        <n-input-number
          v-model:value="ip_model.network_mask"
          min="0"
          max="32"
          placeholder=""
        />
      </n-form-item-gi> -->
      <n-form-item-gi label="分配 IP起始地址" :span="5">
        <NewIpEdit v-model:ip="ip_model.ip_range_start"></NewIpEdit>
      </n-form-item-gi>
      <n-form-item-gi label="分配 IP结束地址" :span="5">
        <NewIpEdit v-model:ip="ip_model.ip_range_end"></NewIpEdit>
      </n-form-item-gi>
    </n-grid>
  </n-form>
</template>
