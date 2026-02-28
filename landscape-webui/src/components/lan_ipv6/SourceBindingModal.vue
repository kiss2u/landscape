<script setup lang="ts">
import { get_all_ipv6pd_status } from "@/api/service_ipv6pd";
import { ServiceStatus } from "@/lib/services";
import type { LanIPv6SourceConfig } from "@landscape-router/types/api/schemas";
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n({ useScope: "global" });

const show = defineModel<boolean>("show", { required: true });
const source = defineModel<LanIPv6SourceConfig>("source");

/** Which service kinds to allow. If empty, all are allowed. */
const props = defineProps<{
  allowedServiceKinds?: ("ra" | "na" | "pd")[];
}>();

const edit_source = ref<LanIPv6SourceConfig>();

type ServiceKind = "ra" | "na" | "pd";
type SourceType = "static" | "pd";

const service_kind = ref<ServiceKind>("ra");
const source_type = ref<SourceType>("static");

const available_service_kinds = computed(() => {
  if (props.allowedServiceKinds && props.allowedServiceKinds.length > 0) {
    return props.allowedServiceKinds;
  }
  return ["ra", "na", "pd"] as ServiceKind[];
});

function tag_for_source(src: LanIPv6SourceConfig): {
  kind: ServiceKind;
  type: SourceType;
} {
  switch (src.t) {
    case "ra_static":
      return { kind: "ra", type: "static" };
    case "ra_pd":
      return { kind: "ra", type: "pd" };
    case "na_static":
      return { kind: "na", type: "static" };
    case "na_pd":
      return { kind: "na", type: "pd" };
    case "pd_static":
      return { kind: "pd", type: "static" };
    case "pd_pd":
      return { kind: "pd", type: "pd" };
  }
}

function make_default(
  kind: ServiceKind,
  type: SourceType,
): LanIPv6SourceConfig {
  if (kind === "ra" && type === "static") {
    return {
      t: "ra_static",
      base_prefix: "fd11:2222:3333:4400::",
      pool_index: 0,
      preferred_lifetime: 300,
      valid_lifetime: 600,
    };
  } else if (kind === "ra" && type === "pd") {
    return {
      t: "ra_pd",
      depend_iface: "",
      pool_index: 0,
      preferred_lifetime: 300,
      valid_lifetime: 600,
    };
  } else if (kind === "na" && type === "static") {
    return {
      t: "na_static",
      base_prefix: "fd11:2222:3333:4400::",
      pool_index: 0,
    };
  } else if (kind === "na" && type === "pd") {
    return {
      t: "na_pd",
      depend_iface: "",
      pool_index: 0,
    };
  } else if (kind === "pd" && type === "static") {
    return {
      t: "pd_static",
      base_prefix: "fd00::",
      base_prefix_len: 48,
      pool_index: 0,
      pool_len: 56,
    };
  } else {
    return {
      t: "pd_pd",
      depend_iface: "",
      max_source_prefix_len: 56,
      pool_index: 0,
      pool_len: 56,
    };
  }
}

function on_service_kind_change(kind: ServiceKind) {
  service_kind.value = kind;
  edit_source.value = make_default(kind, source_type.value);
}

function on_source_type_change(type: SourceType) {
  source_type.value = type;
  edit_source.value = make_default(service_kind.value, type);
}

// PD interface search
const ipv6_pd_ifaces = ref<Map<string, ServiceStatus>>(new Map());
const loading_search_ipv6pd = ref(false);

const ipv6_pd_options = computed(() => {
  const result = [];
  for (const [key, value] of ipv6_pd_ifaces.value) {
    result.push({ value: key, label: `${key} - ${value.t}` });
  }
  return result;
});

async function search_ipv6_pd() {
  ipv6_pd_ifaces.value = await get_all_ipv6pd_status();
}

async function enter() {
  await search_ipv6_pd();
  if (source.value) {
    edit_source.value = JSON.parse(JSON.stringify(source.value));
    const tags = tag_for_source(source.value);
    service_kind.value = tags.kind;
    source_type.value = tags.type;
  } else {
    const kind = available_service_kinds.value[0] ?? "ra";
    service_kind.value = kind;
    source_type.value = "static";
    edit_source.value = make_default(kind, "static");
  }
}

const emit = defineEmits(["commit"]);
async function commit() {
  if (!edit_source.value) return;

  // Validate PD source has depend_iface
  if (
    source_type.value === "pd" &&
    "depend_iface" in edit_source.value &&
    (edit_source.value as any).depend_iface.trim() === ""
  ) {
    window.$message.error(t("lan_ipv6.source_no_iface"));
    return;
  }

  emit("commit", edit_source.value);
  show.value = false;
}
</script>
<template>
  <n-modal
    :auto-focus="false"
    style="width: 600px"
    v-model:show="show"
    class="custom-card"
    preset="card"
    :title="t('lan_ipv6.source_edit_title')"
    size="small"
    :bordered="false"
    @after-enter="enter"
  >
    <template #header-extra>
      <n-flex :gap="8" align="center">
        <n-radio-group
          v-if="edit_source"
          :value="service_kind"
          @update:value="on_service_kind_change"
          name="service-kind"
          size="small"
        >
          <n-radio-button
            v-for="kind in available_service_kinds"
            :key="kind"
            :value="kind"
            :label="t(`lan_ipv6.service_kind_${kind}`)"
          />
        </n-radio-group>
        <n-radio-group
          v-if="edit_source"
          :value="source_type"
          @update:value="on_source_type_change"
          name="source-type"
          size="small"
        >
          <n-radio-button
            value="static"
            :label="t('lan_ipv6.source_type_static')"
          />
          <n-radio-button value="pd" :label="t('lan_ipv6.source_type_pd')" />
        </n-radio-group>
      </n-flex>
    </template>

    <n-flex v-if="edit_source" vertical>
      <!-- Source info: Static prefix or PD interface -->
      <n-grid :x-gap="12" :y-gap="8" cols="4" item-responsive>
        <!-- Static: base_prefix -->
        <template v-if="source_type === 'static'">
          <n-form-item-gi
            v-if="service_kind === 'pd'"
            span="4"
            :label="t('lan_ipv6.source_base_prefix_cidr')"
          >
            <n-flex style="flex: 1" :gap="8">
              <n-input
                style="flex: 3"
                v-model:value="(edit_source as any).base_prefix"
                :placeholder="'fd00::'"
                clearable
              />
              <n-input-number
                style="flex: 1"
                v-model:value="(edit_source as any).base_prefix_len"
                :min="1"
                :max="127"
                :placeholder="'/48'"
              />
            </n-flex>
          </n-form-item-gi>
          <n-form-item-gi
            v-else
            span="4"
            :label="t('lan_ipv6.source_base_prefix')"
          >
            <n-flex style="flex: 1" vertical>
              <n-alert type="warning" :bordered="false" style="font-size: 12px">
                {{ t("lan_ipv6.source_base_prefix_hint") }}
              </n-alert>
              <n-input
                style="flex: 1"
                v-model:value="(edit_source as any).base_prefix"
                :placeholder="'fd11:2222:3333:4400::'"
                clearable
              />
            </n-flex>
          </n-form-item-gi>
        </template>

        <!-- PD: depend_iface -->
        <template v-else>
          <n-form-item-gi span="4" :label="t('lan_ipv6.source_depend_iface')">
            <n-select
              v-model:value="(edit_source as any).depend_iface"
              filterable
              :placeholder="t('lan_ipv6.source_depend_iface_placeholder')"
              :options="ipv6_pd_options"
              :loading="loading_search_ipv6pd"
              clearable
              remote
              @search="search_ipv6_pd"
            />
          </n-form-item-gi>
        </template>

        <!-- Service-specific parameters -->
        <!-- pool_index for all variants -->
        <n-form-item-gi span="2">
          <template #label>
            <Notice>
              {{ t("lan_ipv6.source_pool_index") }}
              <template #msg>
                {{ t("lan_ipv6.source_pool_index_desc") }}
              </template>
            </Notice>
          </template>
          <n-input-number
            style="flex: 1"
            :min="0"
            v-model:value="(edit_source as any).pool_index"
            clearable
          />
        </n-form-item-gi>

        <!-- PD-specific: pool_len -->
        <n-form-item-gi v-if="service_kind === 'pd'" span="2">
          <template #label>
            <Notice>
              {{ t("lan_ipv6.source_pool_len") }}
              <template #msg>
                {{ t("lan_ipv6.source_pool_len_desc") }}
              </template>
            </Notice>
          </template>
          <n-input-number
            style="flex: 1"
            :min="1"
            :max="128"
            v-model:value="(edit_source as any).pool_len"
            clearable
          />
        </n-form-item-gi>

        <!-- PD + PD source: max_source_prefix_len -->
        <n-form-item-gi
          v-if="service_kind === 'pd' && source_type === 'pd'"
          span="2"
        >
          <template #label>
            <Notice>
              {{ t("lan_ipv6.source_max_source_prefix_len") }}
              <template #msg>
                {{ t("lan_ipv6.source_max_source_prefix_len_desc") }}
              </template>
            </Notice>
          </template>
          <n-input-number
            style="flex: 1"
            :min="1"
            :max="126"
            v-model:value="(edit_source as any).max_source_prefix_len"
            clearable
          />
        </n-form-item-gi>

        <!-- RA: preferred_lifetime, valid_lifetime -->
        <template v-if="service_kind === 'ra'">
          <n-form-item-gi span="2">
            <template #label>
              <Notice>
                {{ t("lan_ipv6.source_preferred_lifetime") }}
                <template #msg>
                  {{ t("lan_ipv6.source_preferred_lifetime_desc") }}
                </template>
              </Notice>
            </template>
            <n-input-number
              style="flex: 1"
              v-model:value="(edit_source as any).preferred_lifetime"
              clearable
            />
          </n-form-item-gi>
          <n-form-item-gi span="2">
            <template #label>
              {{ t("lan_ipv6.source_valid_lifetime") }}
            </template>
            <n-input-number
              style="flex: 1"
              v-model:value="(edit_source as any).valid_lifetime"
              clearable
            />
          </n-form-item-gi>
        </template>
      </n-grid>
    </n-flex>

    <template #footer>
      <n-flex justify="space-between">
        <n-button @click="show = false">{{ t("lan_ipv6.cancel") }}</n-button>
        <n-button @click="commit" type="success">{{
          t("lan_ipv6.confirm")
        }}</n-button>
      </n-flex>
    </template>
  </n-modal>
</template>
