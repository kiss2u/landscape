<script setup lang="ts">
import { post_lan_ip_rules } from "@/api/mark";
import { DomainMatchType, RuleSource } from "@/lib/dns";
import { useMessage } from "naive-ui";

import { computed, onMounted } from "vue";
import { ref } from "vue";

import PacketMark from "@/components/mark/PacketMark.vue";
import NewIpEdit from "@/components/NewIpEdit.vue";
import { IpConfig, LanIPRuleConfig } from "@/lib/mark";

const message = useMessage();

const emit = defineEmits(["refresh"]);

const show = defineModel<boolean>("show", { required: true });

const origin_rule = defineModel<LanIPRuleConfig>("rule", {
  default: new LanIPRuleConfig(),
});
const rule = ref<LanIPRuleConfig>(new LanIPRuleConfig(origin_rule.value));

const commit_spin = ref(false);
const isModified = computed(() => {
  return JSON.stringify(rule.value) !== JSON.stringify(origin_rule.value);
});

function onCreate(): IpConfig {
  return new IpConfig();
}

async function saveRule() {
  if (rule.value.index == -1) {
    message.warning("**优先级** 值不能为 -1, 且不能重复, 否则将会覆盖规则");
    return;
  }
  try {
    commit_spin.value = true;
    await post_lan_ip_rules(rule.value);
    console.log("submit success");
    origin_rule.value = rule.value;
    show.value = false;
  } catch (e: any) {
    message.error(`${e.response.data}`);
  } finally {
    commit_spin.value = false;
  }
  emit("refresh");
}

const source_style = [
  {
    label: "精确匹配",
    value: DomainMatchType.Full,
  },
  {
    label: "域名匹配",
    value: DomainMatchType.Domain,
  },
  {
    label: "正则匹配",
    value: DomainMatchType.Regex,
  },
  {
    label: "关键词匹配",
    value: DomainMatchType.Plain,
  },
];
</script>

<template>
  <n-modal
    v-model:show="show"
    style="width: 600px"
    class="custom-card"
    preset="card"
    title="规则编辑"
    :bordered="false"
  >
    <!-- {{ isModified }} -->
    <n-form style="flex: 1" ref="formRef" :model="rule" :cols="5">
      <n-grid :cols="5">
        <n-form-item-gi label="优先级" :span="2">
          <n-input-number v-model:value="rule.index" clearable />
        </n-form-item-gi>
        <n-form-item-gi label="启用" :offset="1" :span="1">
          <n-switch v-model:value="rule.enable">
            <template #checked> 启用 </template>
            <template #unchecked> 禁用 </template>
          </n-switch>
        </n-form-item-gi>

        <n-form-item-gi :span="5" label="流量标记">
          <PacketMark v-model:mark="rule.mark"></PacketMark>
        </n-form-item-gi>
      </n-grid>
      <n-form-item label="备注">
        <n-input v-model:value="rule.remark" type="text" />
      </n-form-item>
      <n-form-item label="匹配的 IP">
        <n-dynamic-input v-model:value="rule.source" :on-create="onCreate">
          <template #create-button-default> 增加一条 Lan 规则 </template>
          <template #default="{ value, index }">
            <NewIpEdit
              v-model:ip="value.ip"
              v-model:mask="value.prefix"
            ></NewIpEdit>
          </template>
        </n-dynamic-input>
      </n-form-item>
    </n-form>
    <template #footer>
      <n-flex justify="space-between">
        <n-button>取消</n-button>
        <n-button
          :loading="commit_spin"
          @click="saveRule"
          :disabled="!isModified"
        >
          保存
        </n-button>
      </n-flex>
    </template>
  </n-modal>
</template>
