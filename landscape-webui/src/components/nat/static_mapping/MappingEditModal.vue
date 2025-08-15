<script setup lang="ts">
import { useMessage } from "naive-ui";
import { StaticNatMappingConfig } from "@/rust_bindings/common/nat";
import { v4 as uuidv4 } from "uuid";

import { computed } from "vue";
import { ref } from "vue";
import {
  copy_context_to_clipboard,
  read_context_from_clipboard,
} from "@/lib/common";
import {
  get_static_nat_mapping,
  push_static_nat_mapping,
} from "@/api/static_nat_mapping";

type Props = {
  rule_id: string | null;
};

const props = defineProps<Props>();

const message = useMessage();

const emit = defineEmits(["refresh"]);

const show = defineModel<boolean>("show", { required: true });

const origin_rule_json = ref<string>("");

const rule = ref<StaticNatMappingConfig>();

const commit_spin = ref(false);
const isModified = computed(() => {
  return JSON.stringify(rule.value) !== origin_rule_json.value;
});

const protocolOptions = [
  { label: "TCP", value: 6 },
  { label: "UDP", value: 17 },
];

const rules = {
  wan_port: [
    {
      required: true,
      type: "number",
      message: "开放端口不能为空",
      trigger: ["blur", "input"],
    },
    {
      validator(rule: any, value: number) {
        if (value <= 0) {
          return new Error("开放端口必须大于 0");
        }
        return true;
      },
      trigger: ["blur", "input"],
    },
  ],
  lan_port: [
    {
      required: true,
      type: "number",
      message: "内网目标端口不能为空",
      trigger: ["blur", "input"],
    },
    {
      validator(rule: any, value: number) {
        if (value <= 0) {
          return new Error("内网目标端口必须大于 0");
        }
        return true;
      },
      trigger: ["blur", "input"],
    },
  ],
  lan_ip: [
    {
      required: true,
      message: "内网目标 IP 不能为空",
      trigger: ["blur", "input"],
    },
  ],
};

async function enter() {
  if (props.rule_id != null) {
    rule.value = await get_static_nat_mapping(props.rule_id);
  } else {
    rule.value = {
      id: uuidv4(),
      enable: true,
      wan_port: 0,
      wan_iface_name: null,
      lan_port: 0,
      lan_ip: "0.0.0.0",
      remark: "",
      l4_protocol: 6,
      update_at: 0,
    };
  }
  origin_rule_json.value = JSON.stringify(rule.value);
}

const formRef = ref();

async function saveRule() {
  if (rule.value) {
    formRef.value?.validate(async (errors: any) => {
      if (errors) return; // 校验不通过，直接退出

      try {
        commit_spin.value = true;
        await push_static_nat_mapping(rule.value!);
        console.log("submit success");
        show.value = false;
        emit("refresh");
      } finally {
        commit_spin.value = false;
      }
    });
  }
}

// async function export_config() {
//   let configs = rule.value.source;
//   await copy_context_to_clipboard(message, JSON.stringify(configs, null, 2));
// }

// async function import_rules() {
//   try {
//     let rules = JSON.parse(await read_context_from_clipboard());
//     rule.value.source = rules;
//   } catch (e) {}
// }
</script>

<template>
  <n-modal
    v-model:show="show"
    style="width: 600px"
    class="custom-card"
    preset="card"
    title="规则编辑"
    @after-enter="enter"
    :bordered="false"
  >
    <!-- {{ isModified }} -->
    <n-form
      v-if="rule"
      :rules="rules"
      style="flex: 1"
      ref="formRef"
      :model="rule"
      :cols="5"
    >
      <n-grid :cols="2">
        <!-- <n-form-item-gi label="优先级" :span="2">
          <n-input-number v-model:value="rule.index" clearable />
        </n-form-item-gi> -->
        <n-form-item-gi label="启用" :span="2">
          <n-switch v-model:value="rule.enable">
            <template #checked> 启用 </template>
            <template #unchecked> 禁用 </template>
          </n-switch>
        </n-form-item-gi>

        <n-form-item-gi label="允许协议" :span="2">
          <n-select
            v-model:value="rule.l4_protocol"
            placeholder="允许协议"
            :options="protocolOptions"
            style="width: 140px"
          />
        </n-form-item-gi>

        <!-- <n-form-item-gi :span="5" label="进入的 wan">
          <n-radio-group v-model:value="rule.wan_iface_name" name="filter">
            <n-radio-button
              v-for="opt in get_dns_filter_options()"
              :key="opt.value"
              :value="opt.value"
              :label="opt.label"
            />
          </n-radio-group>
        </n-form-item-gi> -->

        <n-form-item-gi
          path="wan_port"
          :span="1"
          label="开放端口 (不能与 NAT 映射端口重叠)"
        >
          <n-input-number v-model:value="rule.wan_port" />
        </n-form-item-gi>

        <n-form-item-gi path="lan_port" :span="1" label="内网目标端口">
          <n-input-number v-model:value="rule.lan_port" />
        </n-form-item-gi>

        <n-form-item-gi
          path="lan_ip"
          :span="2"
          label="内网目标 IP ( 如果开放的是路由的端口，那么就设置为 0.0.0.0 或者 [::])"
        >
          <n-input v-model:value="rule.lan_ip" />
        </n-form-item-gi>

        <n-form-item-gi :span="2" label="备注">
          <n-input v-model:value="rule.remark" type="textarea" />
        </n-form-item-gi>
      </n-grid>
    </n-form>
    <template #footer>
      <n-flex justify="space-between">
        <n-button @click="show = false">取消</n-button>
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
