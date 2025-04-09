<script setup lang="ts">
import { PacketMatchMark } from "@/rust_bindings/flow";

const match_rules = defineModel<PacketMatchMark[]>("match_rules", {
  required: true,
});

function onCreate(): PacketMatchMark {
  return {
    mac: "",
    vlan_id: null,
    qos: null,
  };
}
</script>

<template>
  <n-dynamic-input v-model:value="match_rules" :on-create="onCreate">
    <template #create-button-default> 增加一条匹配 </template>
    <template #default="{ value, index }">
      <n-input-group>
        <n-input
          v-model:value="value.mac"
          :style="{ width: '33%' }"
          placeholder="Mac 地址"
        />
        <n-input-number
          v-model:value="value.vlan_id"
          :style="{ width: '33%' }"
          placeholder="VLAN ID"
          :show-button="false"
        />
        <n-input-number
          v-model:value="value.qos"
          min="0"
          max="255"
          :style="{ width: '33%' }"
          placeholder="QoS"
          :show-button="false"
        />
      </n-input-group>
    </template>
  </n-dynamic-input>
</template>
