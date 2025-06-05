<script setup lang="ts">
import { ref } from "vue";
import WanRuleEditModal from "./WanRuleEditModal.vue";
import WanRuleCard from "./WanRuleCard.vue";
import {
  get_flow_dst_ip_rules,
  push_many_dst_ip_rule,
} from "@/api/dst_ip_rule";
import {
  copy_context_to_clipboard,
  read_context_from_clipboard,
} from "@/lib/common";

import { useMessage } from "naive-ui";
const message = useMessage();

interface Props {
  flow_id?: number;
}

const props = withDefaults(defineProps<Props>(), {
  flow_id: 0,
});

const show = defineModel<boolean>("show", { required: true });

const rules = ref<any>([]);
async function read_rules() {
  if (props.flow_id) {
    rules.value = await get_flow_dst_ip_rules(props.flow_id);
  }
}

const show_create_modal = ref(false);

async function export_config() {
  let configs = await get_flow_dst_ip_rules(props.flow_id);
  await copy_context_to_clipboard(
    message,
    JSON.stringify(
      configs,
      (key, value) => {
        if (key === "id") {
          return undefined;
        }
        if (key === "flow_id") {
          return undefined;
        }
        return value;
      },
      2
    )
  );
}

async function import_rules() {
  try {
    let rules = JSON.parse(await read_context_from_clipboard());
    for (const rule of rules) {
      rule.flow_id = props.flow_id;
    }
    await push_many_dst_ip_rule(rules);
    message.success("Import Success");
    await read_rules();
  } catch (e) {}
}
</script>
<template>
  <n-drawer
    @after-enter="read_rules()"
    v-model:show="show"
    width="500px"
    placement="right"
  >
    <n-drawer-content title="编辑 Wan 规则" closable>
      <n-flex style="height: 100%" vertical>
        <n-flex>
          <n-button style="flex: 1" @click="show_create_modal = true">
            增加规则
          </n-button>
          <n-button style="flex: 1" @click="export_config">
            导出规则至剪贴板
          </n-button>
          <n-popconfirm @positive-click="import_rules">
            <template #trigger>
              <n-button style="flex: 1" @click=""> 从剪贴板导入规则 </n-button>
            </template>
            确定从剪贴板导入吗?
          </n-popconfirm>
        </n-flex>

        <n-scrollbar>
          <n-flex vertical>
            <WanRuleCard
              @refresh="read_rules()"
              v-for="rule in rules"
              :key="rule.index"
              :rule="rule"
            >
            </WanRuleCard>
          </n-flex>
        </n-scrollbar>
      </n-flex>

      <WanRuleEditModal
        :id="null"
        :flow_id="flow_id"
        v-model:show="show_create_modal"
        @refresh="read_rules()"
      ></WanRuleEditModal>
    </n-drawer-content>
  </n-drawer>
</template>
