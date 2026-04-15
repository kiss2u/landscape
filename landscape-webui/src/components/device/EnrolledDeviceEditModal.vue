<script setup lang="ts">
import { useMessage } from "naive-ui";
import { isIP, isIPv4 } from "is-ip";
import { computed, ref, watch } from "vue";
import type { EnrolledDevice } from "@landscape-router/types/api/schemas";
import {
  get_enrolled_device_by_id,
  create_enrolled_device,
  update_enrolled_device,
  validate_enrolled_device_ip,
} from "@/api/enrolled_device";
import { get_all_dhcp_v4_status } from "@/api/service_dhcp_v4";
import { useI18n } from "vue-i18n";
import { useEnrolledDeviceStore } from "@/stores/enrolled_device";

const enrolledDeviceStore = useEnrolledDeviceStore();

type Props = {
  rule_id: string | null;
  initialValues?: {
    mac?: string;
    ipv4?: string;
    name?: string;
    iface_name?: string;
  };
};

const props = defineProps<Props>();
const message = useMessage();
const { t } = useI18n();
const emit = defineEmits(["refresh"]);

const show = defineModel<boolean>("show", { required: true });

const origin_rule_json = ref<string>("");
const rule = ref<EnrolledDevice>({
  name: "",
  mac: "",
  tag: [],
});

const commit_spin = ref(false);
const ifaceOptions = ref<{ label: string; value: string }[]>([]);
const ipv4RangeStatus = ref<"success" | "error" | undefined>(undefined);
const ipv4RangeFeedback = ref("");
const ipv4ValidationToken = ref(0);
const enterToken = ref(0);

function isValidMac(value: string) {
  return /^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$/.test(value);
}

function isValidIpv6Suffix(value?: string) {
  if (!value) return true;
  return /^::[\da-fA-F]{1,4}(::?[\da-fA-F]{1,4})*$/.test(value);
}

function normalizeOptionalString(value?: string) {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function buildPayload(): EnrolledDevice {
  return {
    ...rule.value,
    iface_name: normalizeOptionalString(rule.value.iface_name),
    fake_name: normalizeOptionalString(rule.value.fake_name),
    remark: normalizeOptionalString(rule.value.remark),
    ipv4: normalizeOptionalString(rule.value.ipv4),
    ipv6: normalizeOptionalString(rule.value.ipv6),
  };
}

const hasBasicValidity = computed(() => {
  return (
    !!rule.value.name &&
    !!rule.value.mac &&
    isValidMac(rule.value.mac) &&
    (!rule.value.ipv4 || isIP(rule.value.ipv4)) &&
    isValidIpv6Suffix(rule.value.ipv6)
  );
});

const hasValidIpv4Range = computed(() => {
  return ipv4RangeStatus.value !== "error";
});

const isModified = computed(() => {
  return JSON.stringify(rule.value) !== origin_rule_json.value;
});

const canSave = computed(() => {
  return (
    hasBasicValidity.value &&
    hasValidIpv4Range.value &&
    (props.rule_id ? isModified.value : true)
  );
});

function resetIpv4RangeValidation() {
  ipv4RangeStatus.value = undefined;
  ipv4RangeFeedback.value = "";
}

async function syncIpv4RangeValidation() {
  const ipv4 = rule.value.ipv4;
  const ifaceName = rule.value.iface_name;

  if (!show.value || !ipv4 || !ifaceName || !isIPv4(ipv4)) {
    resetIpv4RangeValidation();
    return;
  }

  const token = ++ipv4ValidationToken.value;

  try {
    const isValid = await validate_enrolled_device_ip(ifaceName, ipv4);
    if (token !== ipv4ValidationToken.value) return;

    if (isValid) {
      resetIpv4RangeValidation();
      return;
    }

    ipv4RangeStatus.value = "error";
    ipv4RangeFeedback.value = t("enrolled_device.ipv4_out_of_range", {
      iface: ifaceName,
    });
  } catch (e) {
    if (token !== ipv4ValidationToken.value) return;
    resetIpv4RangeValidation();
    console.error("IP validation failed", e);
  }
}

function exit() {
  enterToken.value += 1;
  ipv4ValidationToken.value += 1;
  commit_spin.value = false;
  origin_rule_json.value = "";
  ifaceOptions.value = [];
  rule.value = {
    name: "",
    mac: "",
    tag: [],
  };
  resetIpv4RangeValidation();
  formRef.value?.restoreValidation?.();
}

async function enter() {
  const token = ++enterToken.value;

  try {
    const [statusMap, fetched] = await Promise.all([
      get_all_dhcp_v4_status(),
      props.rule_id
        ? get_enrolled_device_by_id(props.rule_id)
        : Promise.resolve(null),
    ]);

    if (token !== enterToken.value || !show.value) return;

    ifaceOptions.value = Array.from(statusMap.keys()).map((name) => ({
      label: name,
      value: name,
    }));

    if (fetched) {
      rule.value = fetched;
    } else {
      if (props.rule_id) {
        message.error(t("enrolled_device.load_failed"));
        show.value = false;
        return;
      }

      rule.value = {
        name: props.initialValues?.name ?? "",
        mac: props.initialValues?.mac ?? "",
        tag: [],
        remark: "",
        fake_name: "",
        ipv4: props.initialValues?.ipv4 ?? undefined,
        ipv6: undefined,
        iface_name: props.initialValues?.iface_name ?? undefined,
      };
    }
  } catch (e) {
    if (token !== enterToken.value || !show.value) return;

    console.error("Failed to load enrolled device modal data", e);
    message.error(
      (e as { response?: { data?: string }; message?: string })?.response
        ?.data ||
        (e as { message?: string })?.message ||
        t("enrolled_device.load_failed"),
    );
    show.value = false;
    return;
  }

  if (token !== enterToken.value || !show.value) return;

  origin_rule_json.value = JSON.stringify(rule.value);
  void syncIpv4RangeValidation();
}

const formRef = ref();

const macRule = {
  trigger: ["input", "blur"],
  validator(_: unknown, value: string) {
    if (!value) return new Error(t("enrolled_device.mac_required"));
    if (!isValidMac(value)) return new Error(t("enrolled_device.mac_invalid"));
    return true;
  },
};

watch(
  () => [show.value, rule.value.iface_name, rule.value.ipv4],
  () => {
    void syncIpv4RangeValidation();
  },
);

const ipRule = {
  trigger: ["input", "blur"],
  async validator(_: unknown, value: string) {
    if (value && !isIP(value))
      return new Error(t("enrolled_device.ipv4_invalid"));

    if (value && rule.value.iface_name && isIPv4(value)) {
      try {
        const isValid = await validate_enrolled_device_ip(
          rule.value.iface_name,
          value,
        );
        if (!isValid) {
          return new Error(
            t("enrolled_device.ipv4_out_of_range", {
              iface: rule.value.iface_name,
            }),
          );
        }
      } catch (e) {
        console.error("IP validation failed", e);
      }
    }
    return true;
  },
};

const rules = {
  name: {
    required: true,
    message: t("enrolled_device.name_required"),
    trigger: "blur",
  },
  mac: macRule,
  ipv4: ipRule,
  ipv6: {
    trigger: ["input", "blur"],
    validator(_: unknown, value: string) {
      if (!value) return true;
      if (!isValidIpv6Suffix(value))
        return new Error("请输入有效的 IPv6 后缀 (如 ::100)");
      return true;
    },
  },
};

async function saveRule() {
  await formRef.value?.validate();

  try {
    commit_spin.value = true;
    const payload = buildPayload();

    if (props.rule_id) {
      await update_enrolled_device(props.rule_id, payload);
    } else {
      await create_enrolled_device(payload);
    }
    message.success(t("enrolled_device.save_success"));
    show.value = false;
    await enrolledDeviceStore.UPDATE_INFO();
    emit("refresh");
  } catch (e) {
    console.error(e);
    message.error(
      (e as { response?: { data?: string }; message?: string })?.response
        ?.data ||
        (e as { message?: string })?.message ||
        t("enrolled_device.save_failed"),
    );
  } finally {
    commit_spin.value = false;
  }
}
</script>

<template>
  <n-modal
    :auto-focus="false"
    v-model:show="show"
    style="width: 600px"
    preset="card"
    :title="
      props.rule_id
        ? t('enrolled_device.edit_title')
        : t('enrolled_device.add_title')
    "
    @after-enter="enter"
    @after-leave="exit"
  >
    <n-form
      v-if="rule"
      :rules="rules"
      ref="formRef"
      :model="rule"
      label-placement="left"
      label-width="100"
    >
      <n-grid :cols="2" x-gap="12">
        <n-form-item-gi
          :span="2"
          :label="t('enrolled_device.name')"
          path="name"
        >
          <n-input
            v-model:value="rule.name"
            :placeholder="t('enrolled_device.name_placeholder')"
          />
        </n-form-item-gi>

        <n-form-item-gi :span="2" :label="t('enrolled_device.mac')" path="mac">
          <n-input
            v-model:value="rule.mac"
            :placeholder="t('enrolled_device.mac_placeholder')"
          />
        </n-form-item-gi>

        <n-form-item-gi
          :span="2"
          :label="t('enrolled_device.iface')"
          path="iface_name"
        >
          <n-select
            v-model:value="rule.iface_name"
            :options="ifaceOptions"
            :placeholder="t('enrolled_device.iface_placeholder')"
            clearable
          />
        </n-form-item-gi>

        <n-form-item-gi
          :span="2"
          :label="t('enrolled_device.fake_name')"
          path="fake_name"
        >
          <n-input
            v-model:value="rule.fake_name"
            :placeholder="t('enrolled_device.fake_name_placeholder')"
          />
        </n-form-item-gi>

        <n-form-item-gi
          :label="t('enrolled_device.ipv4')"
          path="ipv4"
          :validation-status="ipv4RangeStatus"
          :feedback="ipv4RangeFeedback"
        >
          <n-input
            v-model:value="rule.ipv4"
            :placeholder="t('enrolled_device.ipv4_placeholder')"
          />
        </n-form-item-gi>

        <n-form-item-gi :label="t('enrolled_device.ipv6')" path="ipv6">
          <n-input
            v-model:value="rule.ipv6"
            :placeholder="t('enrolled_device.ipv6_placeholder')"
          />
        </n-form-item-gi>

        <n-form-item-gi :span="2" :label="t('enrolled_device.tag')" path="tag">
          <n-dynamic-tags v-model:value="rule.tag" />
        </n-form-item-gi>

        <n-form-item-gi
          :span="2"
          :label="t('enrolled_device.remark')"
          path="remark"
        >
          <n-input
            v-model:value="rule.remark"
            type="textarea"
            :placeholder="t('enrolled_device.remark_placeholder')"
          />
        </n-form-item-gi>
      </n-grid>
    </n-form>

    <template #footer>
      <n-flex justify="end">
        <n-space>
          <n-button @click="show = false">{{
            t("enrolled_device.cancel")
          }}</n-button>
          <n-button
            type="primary"
            :loading="commit_spin"
            @click="saveRule"
            :disabled="!canSave"
          >
            {{ t("enrolled_device.save") }}
          </n-button>
        </n-space>
      </n-flex>
    </template>
  </n-modal>
</template>
