<script setup lang="ts">
import { ref } from "vue";
import DnsRuleCard from "@/components/dns/DnsRuleCard.vue";
import { get_flow_dns_rules } from "@/api/dns_rule";

interface Props {
  flow_id?: number;
}

const props = withDefaults(defineProps<Props>(), {
  flow_id: 0,
});

const show = defineModel<boolean>("show", { required: true });
const rules = ref<any>([]);

async function read_rules() {
  rules.value = await get_flow_dns_rules(props.flow_id);
}

const show_create_modal = ref(false);
</script>
<template>
  <n-drawer
    @after-enter="read_rules()"
    v-model:show="show"
    width="500px"
    placement="right"
  >
    <n-drawer-content title="编辑 DNS 所使用规则" closable>
      <n-flex style="height: 100%" vertical>
        <n-button @click="show_create_modal = true">增加规则</n-button>

        <n-scrollbar>
          <n-flex vertical>
            <DnsRuleCard
              @refresh="read_rules()"
              v-for="rule in rules"
              :key="rule.index"
              :rule="rule"
            >
            </DnsRuleCard>
          </n-flex>
        </n-scrollbar>
      </n-flex>

      <DnsRuleEditModal
        v-model:show="show_create_modal"
        :data="{ flow_id: props.flow_id }"
        @refresh="read_rules()"
      ></DnsRuleEditModal>
    </n-drawer-content>
  </n-drawer>
</template>
