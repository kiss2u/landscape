<script setup lang="ts">
import { ref } from "vue";
import { ModelBuilder } from "@vicons/carbon";
import DnsRuleDrawer from "@/components/dns/DnsRuleDrawer.vue";

const emit = defineEmits(["create-flow"]);

const show_dns_rule = ref(false);
const show_ip_rule = ref(false);

async function create_flow() {
  emit("create-flow");
}
</script>

<template>
  <n-card
    style="min-height: 224px"
    size="small"
    title="默认 Flow"
    :hoverable="true"
  >
    <template #header-extra>
      <n-flex>
        <n-button secondary @click="show_dns_rule = true" size="small">
          DNS
        </n-button>
        <n-button secondary @click="show_ip_rule = true" size="small">
          目标 IP
        </n-button>
      </n-flex>
    </template>

    <n-empty>
      <n-flex vertical align="center">
        <n-flex>未被其他 Flow 匹配的流量</n-flex>
        <n-flex>将按默认 Flow 中的规则进行处理</n-flex>
      </n-flex>

      <template #icon>
        <n-icon>
          <ModelBuilder />
        </n-icon>
      </template>
      <template #extra>
        <n-button @click="create_flow" size="small"> 创建一个新 Flow </n-button>
      </template>
    </n-empty>

    <DnsRuleDrawer v-model:show="show_dns_rule" :flow_id="0"> </DnsRuleDrawer>
    <WanIpRuleDrawer v-model:show="show_ip_rule" :flow_id="0" />
  </n-card>
</template>
