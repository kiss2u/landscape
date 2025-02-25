<script setup lang="ts">
import { computed } from "vue";
import { ref } from "vue";
import { useMessage } from "naive-ui";
import { ChangeCatalog } from "@vicons/carbon";

import { post_wan_ip_rules } from "@/api/mark";
import PacketMark from "@/components/mark/PacketMark.vue";
import NewIpEdit from "@/components/NewIpEdit.vue";
import { new_wan_rules, WanIPRuleConfig, WanIPRuleSource } from "@/lib/mark";

const message = useMessage();

const emit = defineEmits(["refresh"]);

const show = defineModel<boolean>("show", { required: true });

const origin_rule = defineModel<WanIPRuleConfig>("rule", {
  default: new WanIPRuleConfig(),
});
const rule = ref<WanIPRuleConfig>(new WanIPRuleConfig(origin_rule.value));

const commit_spin = ref(false);
const isModified = computed(() => {
  return JSON.stringify(rule.value) !== JSON.stringify(origin_rule.value);
});

function onCreate(): WanIPRuleSource {
  return new_wan_rules({ t: "config", ip: "0.0.0.0", prefix: 32 });
}

function changeCurrentRuleType(value: WanIPRuleSource, index: number) {
  if (value.t == "config") {
    rule.value.source[index] = { t: "geokey", key: "" };
  } else {
    rule.value.source[index] = new_wan_rules({
      t: "config",
      ip: "0.0.0.0",
      prefix: 32,
    });
  }
}

async function saveRule() {
  if (rule.value.index == -1) {
    message.warning("**优先级** 值不能为 -1, 且不能重复, 否则将会覆盖规则");
    return;
  }
  try {
    commit_spin.value = true;
    await post_wan_ip_rules(rule.value);
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
</script>

<template>
  <n-modal
    v-model:show="show"
    style="width: 700px"
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
                <NewIpEdit
                  v-model:ip="value.ip"
                  v-model:mask="value.prefix"
                ></NewIpEdit>
              </n-flex>
            </n-flex>
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
