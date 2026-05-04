<script setup lang="ts">
import { useMessage } from "naive-ui";
import type {
  StaticNatMappingConfig,
  StaticNatTarget,
} from "@landscape-router/types/api/schemas";

import { computed, ref } from "vue";
import {
  get_static_nat_mapping,
  push_static_nat_mapping,
} from "@/api/static_nat_mapping";
import { useEnrolledDeviceStore } from "@/stores/enrolled_device";
import { useI18n } from "vue-i18n";

type Props = {
  rule_id?: string;
  initialFocusIndex?: number;
};

const props = defineProps<Props>();

const message = useMessage();
const { t } = useI18n();

const emit = defineEmits(["refresh"]);

const show = defineModel<boolean>("show", { required: true });

const origin_rule_json = ref<string>("");

const rule = ref<StaticNatMappingConfig>();
const portInputRefs = ref<any[]>([]); // Array to store input refs

const enrolledDeviceStore = useEnrolledDeviceStore();
type TargetMode = "address" | "local" | "device";
const targetMode = ref<TargetMode>("device");
const selectedDeviceId = ref<string | null>(null);

const deviceOptions = computed(() =>
  enrolledDeviceStore.bindings.map((d) => ({
    label: `${d.name} (${d.mac})`,
    value: d.id!,
  })),
);

const commit_spin = ref(false);
const isModified = computed(() => {
  return JSON.stringify(rule.value) !== origin_rule_json.value;
});

function legacyTargetFromRule(value: StaticNatMappingConfig): StaticNatTarget {
  return value.lan_target ?? { t: "address" };
}

function syncTargetFormFromRule() {
  if (!rule.value) return;
  const target = rule.value.lan_target ?? legacyTargetFromRule(rule.value);
  targetMode.value = target.t;
  selectedDeviceId.value = target.t === "device" ? target.device_id : null;
}

function syncRuleTarget() {
  if (!rule.value) return;
  if (targetMode.value === "local") {
    rule.value.lan_target = { t: "local" };
    return;
  }
  if (targetMode.value === "device") {
    rule.value.lan_target = selectedDeviceId.value
      ? { t: "device", device_id: selectedDeviceId.value }
      : { t: "device", device_id: "" };
    return;
  }

  rule.value.lan_target = {
    t: "address",
    ipv4:
      rule.value.lan_target?.t === "address"
        ? rule.value.lan_target.ipv4 || undefined
        : undefined,
    ipv6:
      rule.value.lan_target?.t === "address"
        ? rule.value.lan_target.ipv6 || undefined
        : undefined,
  };
}

const addressTarget = computed<StaticNatTarget & { t: "address" }>(() => {
  if (!rule.value?.lan_target || rule.value.lan_target.t !== "address") {
    return { t: "address" };
  }
  return rule.value.lan_target;
});

const selectedDevice = computed(() =>
  enrolledDeviceStore.bindings.find(
    (device) => device.id === selectedDeviceId.value,
  ),
);

const ipv4Pattern =
  /^(25[0-5]|2[0-4]\d|1\d{2}|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d{2}|[1-9]?\d)){3}$/;
const ipv6Pattern =
  /^(([0-9a-fA-F]{1,4}:){7}([0-9a-fA-F]{1,4}|:)|(([0-9a-fA-F]{1,4}:){1,7}:)|(([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4})|(([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2})|(([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3})|(([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4})|(([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5})|([0-9a-fA-F]{1,4}:)((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))$/;

const rules = {};

async function enter() {
  if (props.rule_id) {
    rule.value = await get_static_nat_mapping(props.rule_id);
  } else {
    rule.value = {
      enable: true,
      mapping_pair_ports: [{ wan_port: 0, lan_port: 0 }],
      wan_iface_name: null,
      lan_target: { t: "device", device_id: "" },
      remark: "",
      ipv4_l4_protocol: [6],
      ipv6_l4_protocol: [],
    };
  }
  if (!rule.value.lan_target) {
    rule.value.lan_target = legacyTargetFromRule(rule.value);
  }
  syncTargetFormFromRule();
  origin_rule_json.value = JSON.stringify(rule.value);

  // Handle auto-focus on specific port index
  const focusIdx = props.initialFocusIndex;
  if (focusIdx !== undefined && focusIdx >= 0) {
    // Small delay to ensure rendering is complete
    setTimeout(() => {
      const targetInput = portInputRefs.value[focusIdx];
      if (targetInput) {
        targetInput.focus();
        // Optional: scroll into view if the list is long
        targetInput.$el?.scrollIntoView({
          behavior: "smooth",
          block: "center",
        });
      }
    }, 100);
  }
}

// Functions to manage port pairs
function addPortPair() {
  if (rule.value) {
    rule.value.mapping_pair_ports.push({ wan_port: 0, lan_port: 0 });
    // Focus the new input in next tick
    setTimeout(() => {
      const index = rule.value!.mapping_pair_ports.length - 1;
      const input = portInputRefs.value[index];
      if (input) input.focus();
    }, 100);
  }
}

function removePortPair(index: number) {
  if (rule.value && rule.value.mapping_pair_ports.length > 1) {
    rule.value.mapping_pair_ports.splice(index, 1);
  }
}

const formRef = ref();

async function saveRule() {
  if (rule.value) {
    try {
      // Validate form
      await formRef.value?.validate();

      if (rule.value.lan_target?.t === "address") {
        if (
          rule.value.lan_target.ipv4 &&
          !ipv4Pattern.test(rule.value.lan_target.ipv4)
        ) {
          message.error(t("nat.mapping.validation_ipv4"));
          return;
        }
        if (
          rule.value.lan_target.ipv6 &&
          !ipv6Pattern.test(rule.value.lan_target.ipv6)
        ) {
          message.error(t("nat.mapping.validation_ipv6"));
          return;
        }
      }

      if (
        rule.value.ipv4_l4_protocol.length === 0 &&
        rule.value.ipv6_l4_protocol.length === 0
      ) {
        message.error(t("nat.mapping.select_protocol_required"));
        return;
      }

      if (targetMode.value === "device" && !selectedDeviceId.value) {
        message.error(t("nat.mapping.select_device_required"));
        return;
      }

      if (
        targetMode.value === "device" &&
        rule.value.ipv6_l4_protocol.length > 0 &&
        !selectedDevice.value?.ipv6
      ) {
        message.error(t("nat.mapping.device_ipv6_required"));
        return;
      }

      commit_spin.value = true;
      syncRuleTarget();
      await push_static_nat_mapping(rule.value);
      console.log("submit success");
      show.value = false;
      emit("refresh");
    } catch (e) {
      console.error("Validation failed:", e);
    } finally {
      commit_spin.value = false;
    }
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

const allProtocols = [6, 17]; // Supported protocols
const totalSelectable = allProtocols.length * 2; // 4

const allSelected = computed({
  get() {
    if (!rule.value) return false;
    const selected = [
      ...(rule.value.ipv4_l4_protocol || []),
      ...(rule.value.ipv6_l4_protocol || []),
    ];
    return selected.length === totalSelectable;
  },
  set(val: boolean) {
    if (!rule.value) return;
    if (val) {
      rule.value.ipv4_l4_protocol = [...allProtocols];
      rule.value.ipv6_l4_protocol = [...allProtocols];
    } else {
      rule.value.ipv4_l4_protocol = [];
      rule.value.ipv6_l4_protocol = [];
    }
  },
});

const isIndeterminate = computed(() => {
  if (!rule.value) return false;
  const selected = [
    ...(rule.value.ipv4_l4_protocol || []),
    ...(rule.value.ipv6_l4_protocol || []),
  ];
  return selected.length > 0 && selected.length < totalSelectable;
});

// Port validation rules
const wanPortRule = {
  trigger: ["blur", "input"],
  validator(ruleItem: any, value: number) {
    if (!value && value !== 0) return new Error(t("nat.mapping.required"));
    if (value <= 0 || value > 65535) return new Error(t("nat.mapping.range"));
    return true;
  },
};

const lanPortRule = {
  trigger: ["blur", "input"],
  validator(ruleItem: any, value: number) {
    if (!value && value !== 0) return new Error(t("nat.mapping.required"));
    if (value <= 0 || value > 65535) return new Error(t("nat.mapping.range"));
    return true;
  },
};

// Aggregate list validation rule (for summarized error output)
const mappingPortsRule = {
  trigger: ["change"], // Listen for list changes
  validator(ruleItem: any, value: any[]) {
    // Value can be empty if path binding fails or update is not triggered yet.
    // n-form-item with path="mapping_pair_ports" should provide this array.

    // Fallback to current model when validator value is empty.
    const ports = value || (rule.value ? rule.value.mapping_pair_ports : []);
    if (!ports || ports.length === 0) return true;

    // Collect all validation issues.
    const errors: string[] = [];

    // Check invalid ranges/empty values.
    const hasInvalid = ports.some(
      (p: any) =>
        !p.wan_port ||
        p.wan_port <= 0 ||
        p.wan_port > 65535 ||
        !p.lan_port ||
        p.lan_port <= 0 ||
        p.lan_port > 65535,
    );
    if (hasInvalid) errors.push(t("nat.mapping.invalid_port_value"));

    // Check duplicate mappings.
    const wanPorts = ports.map((p: any) => p.wan_port);
    const hasDuplicateWan = wanPorts.length !== new Set(wanPorts).size;

    const lanPorts = ports.map((p: any) => p.lan_port);
    const hasDuplicateLan = lanPorts.length !== new Set(lanPorts).size;

    if (hasDuplicateWan || hasDuplicateLan) {
      errors.push(t("nat.mapping.duplicate_port_config"));
    }

    if (errors.length > 0) {
      return new Error(errors.join(", "));
    }

    return true;
  },
};
</script>

<template>
  <n-modal
    v-model:show="show"
    style="width: 600px"
    class="custom-card"
    preset="card"
    :title="t('nat.mapping.edit_title')"
    @after-enter="enter"
    :bordered="false"
  >
    <n-flex vertical>
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
          <!-- <n-form-item-gi label="Priority" :span="2">
          <n-input-number v-model:value="rule.index" clearable />
        </n-form-item-gi> -->
          <n-form-item-gi :label="t('nat.mapping.enabled')" :span="2">
            <n-switch v-model:value="rule.enable">
              <template #checked> {{ t("common.enable") }} </template>
              <template #unchecked>
                {{ t("common.disable") }}
              </template>
            </n-switch>
          </n-form-item-gi>

          <n-form-item-gi :label="t('nat.mapping.allowed_protocols')" :span="2">
            <n-flex justify="space-between" style="flex: 1">
              <n-flex>
                <n-checkbox
                  v-model:checked="allSelected"
                  :indeterminate="isIndeterminate"
                >
                  {{ t("nat.mapping.select_all") }}
                </n-checkbox>
              </n-flex>
              <n-flex>
                <n-checkbox-group v-model:value="rule.ipv4_l4_protocol">
                  <n-space item-style="display: flex;">
                    <n-checkbox :value="6" label="TCP v4" />
                    <n-checkbox :value="17" label="UDP v4" />
                  </n-space>
                </n-checkbox-group>
              </n-flex>
              <n-flex>
                <n-checkbox-group v-model:value="rule.ipv6_l4_protocol">
                  <n-space item-style="display: flex;">
                    <n-checkbox :value="6" label="TCP v6" />
                    <n-checkbox :value="17" label="UDP v6" />
                  </n-space>
                </n-checkbox-group>
              </n-flex>
            </n-flex>
          </n-form-item-gi>

          <!-- <n-form-item-gi :span="5" label="Ingress WAN">
          <n-radio-group v-model:value="rule.wan_iface_name" name="filter">
            <n-radio-button
              v-for="opt in get_dns_filter_options()"
              :key="opt.value"
              :value="opt.value"
              :label="opt.label"
            />
          </n-radio-group>
        </n-form-item-gi> -->

          <!-- Port mapping pair list -->
          <n-form-item-gi
            :span="2"
            :label="t('nat.mapping.port_mappings_label')"
            path="mapping_pair_ports"
            :rule="mappingPortsRule"
          >
            <n-flex vertical style="width: 100%; gap: 8px">
              <n-flex
                v-for="(pair, index) in rule.mapping_pair_ports"
                :key="index"
                align="center"
                style="gap: 8px"
              >
                <n-form-item
                  style="flex: 1; margin-bottom: 0"
                  :show-label="false"
                  :show-feedback="false"
                  :path="`mapping_pair_ports[${index}].wan_port`"
                  :rule="wanPortRule"
                >
                  <n-input-number
                    :ref="
                      (el: any) => {
                        if (el) portInputRefs[index] = el;
                      }
                    "
                    v-model:value="pair.wan_port"
                    :min="1"
                    :max="65535"
                    :placeholder="t('nat.mapping.public_port_placeholder')"
                    style="width: 100%"
                  />
                </n-form-item>
                <span style="color: #999">→</span>
                <n-form-item
                  style="flex: 1; margin-bottom: 0"
                  :show-label="false"
                  :show-feedback="false"
                  :path="`mapping_pair_ports[${index}].lan_port`"
                  :rule="lanPortRule"
                >
                  <n-input-number
                    v-model:value="pair.lan_port"
                    :min="1"
                    :max="65535"
                    :placeholder="t('nat.mapping.private_port_placeholder')"
                    style="width: 100%"
                  />
                </n-form-item>
                <n-button
                  v-if="rule.mapping_pair_ports.length > 1"
                  size="small"
                  @click="removePortPair(index)"
                  secondary
                  type="error"
                >
                  {{ t("nat.mapping.delete") }}
                </n-button>
              </n-flex>
              <n-button @click="addPortPair" dashed block size="small">
                {{ t("nat.mapping.add_port_pair") }}
              </n-button>
            </n-flex>
          </n-form-item-gi>

          <n-form-item-gi :span="2" :label="t('nat.mapping.target_type')">
            <n-radio-group
              v-model:value="targetMode"
              @update:value="syncRuleTarget"
            >
              <n-radio-button value="device">
                {{ t("nat.mapping.target_type_device") }}
              </n-radio-button>
              <n-radio-button value="local">
                {{ t("nat.mapping.target_type_local") }}
              </n-radio-button>
              <n-radio-button value="address">
                {{ t("nat.mapping.target_type_address") }}
              </n-radio-button>
            </n-radio-group>
          </n-form-item-gi>

          <n-form-item-gi
            v-if="targetMode === 'address'"
            :span="2"
            :label="t('nat.mapping.target_ipv4')"
          >
            <n-input
              :placeholder="t('nat.mapping.target_ipv4_hint')"
              :value="addressTarget.ipv4 || null"
              @update:value="
                (v: string | null) => {
                  if (rule) {
                    rule.lan_target = {
                      t: 'address',
                      ipv4: v || undefined,
                      ipv6: addressTarget.ipv6 || undefined,
                    };
                    syncRuleTarget();
                  }
                }
              "
            />
          </n-form-item-gi>

          <n-form-item-gi
            v-if="targetMode === 'address'"
            :span="2"
            :label="t('nat.mapping.target_ipv6')"
          >
            <n-input
              :placeholder="t('nat.mapping.target_ipv6_hint')"
              :value="addressTarget.ipv6 || null"
              @update:value="
                (v: string | null) => {
                  if (rule) {
                    rule.lan_target = {
                      t: 'address',
                      ipv4: addressTarget.ipv4 || undefined,
                      ipv6: v || undefined,
                    };
                    syncRuleTarget();
                  }
                }
              "
            />
          </n-form-item-gi>

          <n-form-item-gi
            v-if="targetMode === 'local'"
            :span="2"
            :label="t('nat.mapping.target_local')"
          >
            <n-alert type="info" :show-icon="false" style="width: 100%">
              {{ t("nat.mapping.target_local_hint") }}
            </n-alert>
          </n-form-item-gi>

          <n-form-item-gi
            v-if="targetMode === 'device'"
            :span="2"
            :label="t('nat.mapping.target_device')"
          >
            <n-flex vertical style="width: 100%">
              <n-select
                v-model:value="selectedDeviceId"
                :options="deviceOptions"
                :placeholder="t('nat.mapping.select_device_placeholder')"
                clearable
                filterable
                @update:value="syncRuleTarget"
              />
              <n-text v-if="selectedDevice" depth="3">
                {{ selectedDevice.iface_name || "-" }} /
                {{ selectedDevice.ipv4 || "-" }} /
                {{ selectedDevice.ipv6 || "-" }}
              </n-text>
            </n-flex>
          </n-form-item-gi>

          <n-form-item-gi :span="2" :label="t('nat.mapping.remark')">
            <n-input v-model:value="rule.remark" type="textarea" />
          </n-form-item-gi>
        </n-grid>
      </n-form>
    </n-flex>

    <template #footer>
      <n-flex justify="space-between">
        <n-button @click="show = false">{{ t("common.cancel") }}</n-button>
        <n-button
          :loading="commit_spin"
          @click="saveRule"
          :disabled="!isModified"
        >
          {{ t("common.save") }}
        </n-button>
      </n-flex>
    </template>
  </n-modal>
</template>
