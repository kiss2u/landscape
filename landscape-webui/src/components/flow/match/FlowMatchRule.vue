<script setup lang="ts">
import { computed, reactive } from "vue";
import type { FlowEntryRule } from "@landscape-router/types/api/schemas";
import { useFrontEndStore } from "@/stores/front_end_config";
import { useEnrolledDeviceStore } from "@/stores/enrolled_device";
import { ChangeCatalog } from "@vicons/carbon";
import { is_ipv4, is_ipv6 } from "@/lib/common";
import { formatMacAddress } from "@/lib/util";
import { useI18n } from "vue-i18n";

const frontEndStore = useFrontEndStore();
const enrolledDeviceStore = useEnrolledDeviceStore();
const { t } = useI18n();
const match_rules = defineModel<FlowEntryRule[]>("match_rules", {
  required: true,
});

type InputMode = "device" | "mac" | "ip";
const inputModes = reactive(new Map<number, InputMode>());

function getInputMode(index: number): InputMode {
  return (
    inputModes.get(index) ??
    (match_rules.value[index]?.mode.t === "ip"
      ? "ip"
      : match_rules.value[index]?.mode.t === "device"
        ? "device"
        : "mac")
  );
}

const deviceOptions = computed(() =>
  enrolledDeviceStore.bindings.map((d) => ({
    label: d.name,
    value: d.id!,
  })),
);

function onCreate(): FlowEntryRule {
  const index = match_rules.value.length;
  inputModes.set(index, "device");
  return {
    qos: null,
    mode: {
      t: "device",
      device_id: "",
    },
  };
}

function getDefaultPrefixLen(ip: string): number {
  if (is_ipv6(ip) || ip.includes(":")) {
    return 128;
  }
  if (is_ipv4(ip)) {
    return 32;
  }
  return 32;
}

function change_mode(value: FlowEntryRule, index: number) {
  const current = getInputMode(index);
  const temp_rule = match_rules.value[index];
  if (current === "device") {
    inputModes.set(index, "mac");
    match_rules.value[index] = {
      qos: temp_rule.qos,
      mode: {
        t: "mac",
        mac_addr: "",
      },
    };
  } else if (current === "mac") {
    inputModes.set(index, "ip");
    match_rules.value[index] = {
      qos: temp_rule.qos,
      mode: {
        t: "ip",
        ip: "",
        prefix_len: 32,
      },
    };
  } else {
    inputModes.set(index, "device");
    match_rules.value[index] = {
      qos: temp_rule.qos,
      mode: {
        t: "device",
        device_id: "",
      },
    };
  }
}
</script>

<template>
  <n-dynamic-input v-model:value="match_rules" :on-create="onCreate">
    <template #create-button-default>
      {{ t("flow.match_rule.add_entry_rule") }}
    </template>
    <template #default="{ value, index }">
      <n-flex style="flex: 1" :wrap="false">
        <n-button @click="change_mode(value, index)">
          <n-icon>
            <ChangeCatalog />
          </n-icon>
        </n-button>

        <n-select
          v-if="getInputMode(index) === 'device'"
          :options="deviceOptions"
          :value="
            value.mode.t === 'device' ? value.mode.device_id || null : null
          "
          @update:value="
            (v: string) => {
              value.mode = { t: 'device', device_id: v };
            }
          "
          :placeholder="t('flow.match_rule.select_device_placeholder')"
          clearable
          filterable
          :style="{ minWidth: '140px', flex: 1 }"
        />
        <n-input
          v-else-if="getInputMode(index) === 'mac'"
          :type="frontEndStore.presentation_mode ? 'password' : 'text'"
          :value="value.mode.t === 'mac' ? value.mode.mac_addr : ''"
          @update:value="
            (v: string) => {
              value.mode = { t: 'mac', mac_addr: formatMacAddress(v) };
            }
          "
          :placeholder="t('flow.match_rule.mac_placeholder')"
        />
        <n-input-group v-else>
          <n-input
            :type="frontEndStore.presentation_mode ? 'password' : 'text'"
            :value="value.mode.t === 'ip' ? value.mode.ip : ''"
            @update:value="
              (v: string) => {
                const prefixLen =
                  value.mode.t === 'ip'
                    ? value.mode.ip === '' ||
                      value.mode.prefix_len ===
                        getDefaultPrefixLen(value.mode.ip)
                      ? getDefaultPrefixLen(v)
                      : value.mode.prefix_len
                    : getDefaultPrefixLen(v);
                value.mode = { t: 'ip', ip: v, prefix_len: prefixLen };
              }
            "
            :placeholder="t('flow.match_rule.ip_placeholder')"
          />
          <n-input-group-label>/</n-input-group-label>
          <n-input-number
            :value="
              value.mode.t === 'ip'
                ? value.mode.prefix_len
                : getDefaultPrefixLen('')
            "
            @update:value="
              (v: number | null) => {
                const ip = value.mode.t === 'ip' ? value.mode.ip : '';
                value.mode = {
                  t: 'ip',
                  ip,
                  prefix_len: v ?? getDefaultPrefixLen(ip),
                };
              }
            "
            :style="{ width: '60px' }"
            :placeholder="t('flow.match_rule.prefix_placeholder')"
            :show-button="false"
          />
        </n-input-group>
      </n-flex>
    </template>
  </n-dynamic-input>
</template>
