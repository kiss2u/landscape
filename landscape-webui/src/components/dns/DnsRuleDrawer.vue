<script setup lang="ts">
import { ref } from "vue";
import DnsRuleCard from "@/components/dns/DnsRuleCard.vue";
import { get_dns_rule } from "@/api/dns_service";

const show = defineModel<boolean>("show", { required: true });
const rules = ref<any>([]);

async function read_rules() {
  rules.value = await get_dns_rule();
}

const show_create_modal = ref(false);
</script>
<template>
  <n-drawer
    @after-enter="read_rules"
    v-model:show="show"
    :width="502"
    placement="right"
  >
    <n-drawer-content title="编辑 DNS 所使用规则">
      <n-button @click="show_create_modal = true">增加规则</n-button>

      <n-flex vertical>
        <DnsRuleCard v-for="rule in rules" :key="rule.index" :rule="rule">
        </DnsRuleCard>
      </n-flex>

      <DnsRuleEditModal v-model:show="show_create_modal"></DnsRuleEditModal>
    </n-drawer-content>
  </n-drawer>
</template>
