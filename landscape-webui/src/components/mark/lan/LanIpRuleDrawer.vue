<script setup lang="ts">
import { ref } from "vue";
import LanRuleEditModal from "./LanRuleEditModal.vue";
import LanRuleCard from "./LanRuleCard.vue";
import { get_lan_ip_rules } from "@/api/mark";

const show = defineModel<boolean>("show", { required: true });

const rules = ref<any>([]);
async function read_rules() {
  rules.value = await get_lan_ip_rules();
}

const show_create_modal = ref(false);
</script>
<template>
  <n-drawer
    @after-enter="read_rules()"
    v-model:show="show"
    :width="502"
    placement="right"
  >
    <n-drawer-content title="编辑 LAN 规则">
      <n-flex style="height: 100%" vertical>
        <!-- <n-alert type="warning"> 规则编辑后 </n-alert> -->
        <n-button @click="show_create_modal = true">增加规则</n-button>

        <n-scrollbar>
          <n-flex vertical>
            <LanRuleCard
              @refresh="read_rules()"
              v-for="rule in rules"
              :key="rule.index"
              :rule="rule"
            >
            </LanRuleCard>
          </n-flex>
        </n-scrollbar>
      </n-flex>

      <LanRuleEditModal
        v-model:show="show_create_modal"
        @refresh="read_rules()"
      ></LanRuleEditModal>
    </n-drawer-content>
  </n-drawer>
</template>
