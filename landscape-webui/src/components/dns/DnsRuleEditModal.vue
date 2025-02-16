<script setup lang="ts">
import { push_dns_rule } from "@/api/dns_service";
import { DnsRule, DomainMatchType, RuleSource } from "@/lib/dns";

import { ChangeCatalog } from "@vicons/carbon";
import { computed, onMounted } from "vue";
import { ref } from "vue";

import PacketMark from "@/components/mark/PacketMark.vue";

const show = defineModel<boolean>("show", { required: true });

const origin_rule = defineModel<DnsRule>("rule", { default: new DnsRule() });
const rule = ref<any>(new DnsRule(origin_rule.value));

const isModified = computed(() => {
  return JSON.stringify(rule.value) !== JSON.stringify(origin_rule.value);
});

function onCreate(): RuleSource {
  return {
    t: "geokey",
    key: "",
  };
}

function changeCurrentRuleType(value: RuleSource, index: number) {
  if (value.t == "geokey") {
    rule.value.source[index] = {
      t: "config",
      match_type: DomainMatchType.Full,
      value: value.key,
    };
  } else {
    rule.value.source[index] = { t: "geokey", key: value.value };
  }
}

async function saveRule() {
  await push_dns_rule(rule.value);
  console.log("submit success");
  origin_rule.value = rule.value;
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
    size="huge"
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
        <n-form-item-gi label="重定向" :span="1">
          <n-switch v-model:value="rule.redirection">
            <template #checked> 启用 </template>
            <template #unchecked> 禁用 </template>
          </n-switch>
        </n-form-item-gi>

        <n-form-item-gi :span="5" label="流量标记">
          <!-- <n-popover trigger="hover">
            <template #trigger>
              <n-switch v-model:value="rule.mark">
                <template #checked> 标记 </template>
                <template #unchecked> 不标记 </template>
              </n-switch>
            </template>
            <span>向上游 DNS 请求时的流量是否标记</span>
          </n-popover> -->
          <PacketMark v-model:mark="rule.mark"></PacketMark>
        </n-form-item-gi>
      </n-grid>

      <n-form-item label="名称">
        <n-input v-model:value="rule.name" type="text" />
      </n-form-item>

      <n-form-item label="上游 DNS">
        <n-input
          placeholder="默认使用: 1.1.1.1"
          v-model:value="rule.dns_resolve_ip"
          type="text"
        />
      </n-form-item>
      <n-form-item label="匹配规则 (为空则全部匹配, 规则不分先后)">
        <n-dynamic-input v-model:value="rule.source" :on-create="onCreate">
          <template #create-button-default> 增加一条规则来源 </template>
          <template #default="{ value, index }">
            <n-flex style="flex: 1" :wrap="false">
              <n-button @click="changeCurrentRuleType(value, index)">
                <n-icon>
                  <ChangeCatalog />
                </n-icon>
              </n-button>
              <n-input
                v-if="value.t === 'geokey'"
                v-model:value="value.key"
                placeholder="geo key"
                type="text"
              />
              <n-flex v-else style="flex: 1">
                <n-input-group>
                  <n-select
                    style="width: 38%"
                    v-model:value="value.match_type"
                    :options="source_style"
                    placeholder="选择匹配方式"
                  />
                  <n-input
                    placeholder=""
                    v-model:value="value.value"
                    type="text"
                  />
                </n-input-group>
              </n-flex>
            </n-flex>
          </template>
        </n-dynamic-input>
      </n-form-item>
    </n-form>
    <template #footer>
      <n-flex justify="space-between">
        <n-button>取消</n-button>
        <n-button @click="saveRule" :disabled="!isModified">保存</n-button>
      </n-flex>
    </template>
  </n-modal>
</template>
