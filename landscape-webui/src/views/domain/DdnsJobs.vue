<script lang="ts" setup>
import { get_wan_ifaces } from "@/api/iface";
import {
  delete_ddns_job,
  get_ddns_job_status,
  get_ddns_jobs,
  push_ddns_job,
} from "@/api/domain/ddns";
import { get_dns_provider_profiles } from "@/api/domain/provider_profile";
import type {
  DdnsJob,
  DdnsJobRuntime,
  DdnsRecordConfig,
  DnsProviderProfile,
  IpFamily,
} from "@landscape-router/types/api/schemas";
import { computed, h, onMounted, ref } from "vue";
import { NButton, NPopconfirm, NTag, type DataTableColumns } from "naive-ui";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const items = ref<DdnsJob[]>([]);
const runtimeMap = ref<Map<string, DdnsJobRuntime>>(new Map());
const providerProfiles = ref<DnsProviderProfile[]>([]);
const ifaceOptions = ref<{ label: string; value: string }[]>([]);
type SourceInputItem =
  | { kind: "wan"; target_id: string; family: IpFamily }
  | { kind: "lan_device"; target_id: string; family: "ipv6" };
const loading = ref(false);
const showModal = ref(false);
const saving = ref(false);
const editingId = ref<string | null>(null);
const formRef = ref();
const recordInputs = ref<string[]>([]);
const sourceInputs = ref<SourceInputItem[]>([]);
const form = ref<DdnsJob>({
  name: "",
  enable: true,
  sources: [],
  zone_name: "",
  provider_profile_id: "",
  ttl: 120,
  records: [],
});

const familyOptions = [
  { label: "IPv4", value: "ipv4" },
  { label: "IPv6", value: "ipv6" },
];
const sourceKindOptions = computed(() => [
  { label: t("cert.source_kind_wan"), value: "wan" },
  {
    label: t("cert.source_kind_lan_device"),
    value: "lan_device",
    disabled: true,
  },
]);

const rules = {
  name: {
    required: true,
    message: () => t("cert.job_name_required"),
    trigger: ["input", "blur"],
  },
  zone_name: {
    required: true,
    message: () => t("cert.zone_name_required"),
    trigger: ["input", "blur"],
  },
  provider_profile_id: {
    required: true,
    message: () => t("cert.provider_profile_required"),
    trigger: ["change", "blur"],
  },
};

const providerOptions = computed(() =>
  providerProfiles.value.map((item) => ({ label: item.name, value: item.id! })),
);

function resetForm(item?: DdnsJob) {
  form.value = item
    ? JSON.parse(JSON.stringify(item))
    : {
        name: "",
        enable: true,
        sources: [],
        zone_name: "",
        provider_profile_id: providerProfiles.value[0]?.id ?? "",
        ttl: 120,
        records: [],
      };
  editingId.value = item?.id ?? null;
  sourceInputs.value = item?.sources?.map((source) =>
    source.t === "local_wan"
      ? {
          kind: "wan" as const,
          target_id: source.iface_name,
          family: source.family,
        }
      : {
          kind: "lan_device" as const,
          target_id: source.device_id,
          family: "ipv6" as const,
        },
  ) ?? [
    {
      kind: "wan",
      target_id: ifaceOptions.value[0]?.value ?? "",
      family: "ipv6" as IpFamily,
    },
  ];
  recordInputs.value = item?.records.map((record) => record.name) ?? ["@"];
}

async function refresh() {
  loading.value = true;
  try {
    const [jobs, runtimeStatuses, profiles, wanIfaces] = await Promise.all([
      get_ddns_jobs(),
      get_ddns_job_status(),
      get_dns_provider_profiles(),
      get_wan_ifaces(),
    ]);
    items.value = jobs;
    runtimeMap.value = new Map(
      runtimeStatuses.map((item) => [item.job_id, item]),
    );
    providerProfiles.value = profiles;
    ifaceOptions.value = wanIfaces.map((item: any) => ({
      label: item.name,
      value: item.name,
    }));
  } finally {
    loading.value = false;
  }
}

function providerName(id: string) {
  return providerProfiles.value.find((item) => item.id === id)?.name ?? id;
}

function sourceLabel(job: DdnsJob) {
  return job.sources
    .map((source) =>
      source.t === "local_wan"
        ? `${source.iface_name} / ${source.family.toUpperCase()}`
        : `${t("cert.source_kind_lan_device")} / ${source.device_id}`,
    )
    .join(", ");
}

function statusType(status?: string) {
  switch (status) {
    case "success":
      return "success";
    case "error":
      return "error";
    case "syncing":
      return "warning";
    default:
      return "default";
  }
}

function recordsSummary(job: DdnsJob) {
  return job.records.map((record) => record.name).join(", ");
}

function aggregateStatus(job: DdnsJob) {
  const runtime = runtimeMap.value.get(job.id!);
  const enabledRecords = runtime?.records ?? [];
  if (enabledRecords.length === 0) return "idle";
  const allStatuses = enabledRecords.flatMap((record) => [
    record.ipv4.status,
    record.ipv6.status,
  ]);
  if (allStatuses.some((status) => status === "error")) return "error";
  if (allStatuses.some((status) => status === "syncing")) return "syncing";
  if (
    allStatuses.every(
      (status) => status === "success" || status === "idle" || status == null,
    )
  )
    return "success";
  return "idle";
}

function renderFamilyStatus(status?: string) {
  return h(
    NTag,
    { size: "small", type: statusType(status) },
    () => status ?? "idle",
  );
}

function formatIp(ip?: string | null) {
  return ip || "-";
}

function formatError(err?: string | null) {
  return err || "-";
}

function expandedRowRender(row: DdnsJob) {
  const runtime = runtimeMap.value.get(row.id!);
  return h("div", { style: "padding: 8px 0;" }, [
    h("table", { class: "ddns-detail-table" }, [
      h("thead", {}, [
        h("tr", {}, [
          h("th", {}, t("cert.record_name")),
          h("th", {}, "IPv4"),
          h("th", {}, "IPv4 IP"),
          h("th", {}, "IPv4 Error"),
          h("th", {}, "IPv6"),
          h("th", {}, "IPv6 IP"),
          h("th", {}, "IPv6 Error"),
        ]),
      ]),
      h(
        "tbody",
        {},
        (
          runtime?.records ??
          row.records.map((record) => ({
            name: record.name,
            ipv4: {
              status: "idle",
              last_published_ip: null,
              last_error: null,
              last_sync_at: null,
            },
            ipv6: {
              status: "idle",
              last_published_ip: null,
              last_error: null,
              last_sync_at: null,
            },
          }))
        ).map((record) =>
          h("tr", { key: record.name }, [
            h("td", {}, record.name),
            h("td", {}, [renderFamilyStatus(record.ipv4.status)]),
            h("td", {}, formatIp(record.ipv4.last_published_ip)),
            h("td", {}, formatError(record.ipv4.last_error)),
            h("td", {}, [renderFamilyStatus(record.ipv6.status)]),
            h("td", {}, formatIp(record.ipv6.last_published_ip)),
            h("td", {}, formatError(record.ipv6.last_error)),
          ]),
        ),
      ),
    ]),
  ]);
}

function mergeRecordItems(records: string[], existing: DdnsRecordConfig[]) {
  const byKey = new Map(
    existing.map((record) => [record.name.toLowerCase(), record]),
  );
  return records.map((name) => {
    const old = byKey.get(name.toLowerCase());
    return old
      ? { ...old, name }
      : {
          name,
          enable: true,
        };
  });
}

function normalizeRecordInput(value: string) {
  const normalized = value.trim();
  if (normalized.toLowerCase() === "root") {
    return "@";
  }
  return normalized;
}

function updateRecordInput(index: number, value: string) {
  recordInputs.value[index] = value;
}

function updateSourceKind(index: number, value: "wan" | "lan_device") {
  if (value === "lan_device") {
    sourceInputs.value[index] = {
      kind: "lan_device",
      target_id: "",
      family: "ipv6",
    };
  } else {
    sourceInputs.value[index] = {
      kind: "wan",
      target_id: ifaceOptions.value[0]?.value ?? "",
      family: sourceInputs.value[index]?.family === "ipv4" ? "ipv4" : "ipv6",
    };
  }
}

function updateSourceTarget(index: number, value: string) {
  sourceInputs.value[index].target_id = value;
}

function updateSourceFamily(index: number, value: IpFamily) {
  if (sourceInputs.value[index].kind === "wan") {
    sourceInputs.value[index].family = value;
  }
}

async function save() {
  await formRef.value?.validate();
  saving.value = true;
  try {
    const recordNames = recordInputs.value
      .map((item) => normalizeRecordInput(item))
      .filter(Boolean);

    const sources = sourceInputs.value
      .filter((item) => item.kind === "wan" && item.target_id)
      .map((item) => {
        if (item.kind === "wan") {
          return {
            t: "local_wan" as const,
            iface_name: item.target_id,
            family: item.family,
          };
        }
        return {
          t: "enrolled_device" as const,
          device_id: item.target_id,
          family: "ipv6" as const,
        };
      });

    if (recordNames.length === 0) {
      throw new Error(t("cert.record_name_required"));
    }
    if (sources.length === 0) {
      throw new Error(t("cert.source_required"));
    }

    await push_ddns_job({
      ...form.value,
      id: editingId.value ?? undefined,
      sources,
      ttl: form.value.ttl || null,
      records: mergeRecordItems(recordNames, form.value.records ?? []),
    });
    showModal.value = false;
    await refresh();
  } finally {
    saving.value = false;
  }
}

async function remove(id: string) {
  await delete_ddns_job(id);
  await refresh();
}

const columns = computed<DataTableColumns<DdnsJob>>(() => [
  {
    type: "expand",
    expandable: () => true,
    renderExpand: expandedRowRender,
  },
  { title: t("cert.job_name"), key: "name", minWidth: 120 },
  { title: t("cert.zone_name"), key: "zone_name", minWidth: 160 },
  {
    title: t("cert.records"),
    key: "records",
    minWidth: 220,
    render: (row) => recordsSummary(row) || "-",
  },
  {
    title: t("cert.source"),
    key: "source",
    minWidth: 140,
    render: (row) => sourceLabel(row),
  },
  {
    title: t("cert.provider_profile"),
    key: "provider_profile_id",
    minWidth: 140,
    render: (row) => providerName(row.provider_profile_id),
  },
  {
    title: t("common.enable"),
    key: "enable",
    width: 90,
    render: (row) =>
      h(
        NTag,
        { size: "small", type: row.enable ? "success" : "default" },
        () => (row.enable ? t("common.enable") : t("common.disable")),
      ),
  },
  {
    title: t("common.status"),
    key: "status",
    width: 100,
    render: (row) =>
      h(NTag, { size: "small", type: statusType(aggregateStatus(row)) }, () =>
        aggregateStatus(row),
      ),
  },
  {
    title: t("common.status"),
    key: "actions",
    width: 180,
    render: (row) => [
      h(
        NButton,
        {
          size: "small",
          secondary: true,
          onClick: () => {
            resetForm(row);
            showModal.value = true;
          },
        },
        () => t("common.edit"),
      ),
      h(
        NPopconfirm,
        { onPositiveClick: () => remove(row.id!) },
        {
          trigger: () =>
            h(
              NButton,
              {
                size: "small",
                type: "error",
                secondary: true,
                style: "margin-left: 8px",
              },
              () => t("common.delete"),
            ),
          default: () => t("common.confirm_delete"),
        },
      ),
    ],
  },
]);

onMounted(async () => {
  await refresh();
  if (!form.value.provider_profile_id && providerProfiles.value.length > 0) {
    form.value.provider_profile_id = providerProfiles.value[0].id!;
  }
  if (sourceInputs.value.length === 0) {
    sourceInputs.value = [
      {
        kind: "wan",
        target_id: ifaceOptions.value[0]?.value ?? "",
        family: "ipv6" as IpFamily,
      },
    ];
  }
});
</script>

<template>
  <n-flex vertical style="flex: 1">
    <n-flex justify="space-between">
      <n-button
        @click="
          resetForm();
          showModal = true;
        "
        >{{ t("common.create") }}</n-button
      >
      <n-button :loading="loading" @click="refresh">{{
        t("common.refresh")
      }}</n-button>
    </n-flex>

    <n-data-table
      :columns="columns"
      :data="items"
      :bordered="false"
      :single-line="false"
    />

    <n-modal
      v-model:show="showModal"
      preset="card"
      style="width: 680px"
      :title="t('cert.ddns_jobs')"
    >
      <n-form
        ref="formRef"
        :model="form"
        :rules="rules"
        label-placement="left"
        label-width="auto"
      >
        <n-form-item :label="t('cert.job_name')" path="name"
          ><n-input v-model:value="form.name"
        /></n-form-item>
        <n-form-item :label="t('cert.zone_name')" path="zone_name">
          <n-input v-model:value="form.zone_name" placeholder="example.com" />
        </n-form-item>
        <n-form-item :label="t('cert.records')">
          <n-dynamic-input v-model:value="recordInputs" :min="1">
            <template #default="{ value, index }">
              <n-input
                :value="value"
                :placeholder="t('cert.record_names_placeholder')"
                @update:value="updateRecordInput(index, $event)"
              />
            </template>
          </n-dynamic-input>
        </n-form-item>
        <n-form-item :label="t('cert.sources')">
          <n-dynamic-input v-model:value="sourceInputs" :min="1">
            <template #default="{ value, index }">
              <n-flex style="width: 100%" :size="8" :wrap="false">
                <n-select
                  style="width: 140px"
                  :value="value.kind"
                  :options="sourceKindOptions"
                  @update:value="updateSourceKind(index, $event)"
                />
                <n-select
                  style="flex: 1"
                  :value="value.target_id"
                  :options="ifaceOptions"
                  @update:value="updateSourceTarget(index, $event)"
                />
                <n-select
                  style="width: 120px"
                  :value="value.family"
                  :options="
                    value.kind === 'wan'
                      ? familyOptions
                      : [{ label: 'IPv6', value: 'ipv6' }]
                  "
                  @update:value="updateSourceFamily(index, $event)"
                />
              </n-flex>
            </template>
          </n-dynamic-input>
        </n-form-item>
        <n-form-item
          :label="t('cert.provider_profile')"
          path="provider_profile_id"
        >
          <n-select
            v-model:value="form.provider_profile_id"
            :options="providerOptions"
          />
        </n-form-item>
        <n-form-item :label="t('cert.ttl')">
          <n-input-number
            v-model:value="form.ttl"
            :min="1"
            :precision="0"
            style="width: 100%"
          />
        </n-form-item>
        <n-form-item :label="t('common.enable')">
          <n-switch v-model:value="form.enable" />
        </n-form-item>
      </n-form>

      <n-alert type="info" :show-icon="false" style="margin-top: 8px">
        {{ t("cert.zone_records_hint") }}
      </n-alert>

      <template #footer>
        <n-flex justify="space-between">
          <n-button @click="showModal = false">{{
            t("common.cancel")
          }}</n-button>
          <n-button type="primary" :loading="saving" @click="save">{{
            t("common.save")
          }}</n-button>
        </n-flex>
      </template>
    </n-modal>
  </n-flex>
</template>

<style scoped>
.ddns-detail-table {
  width: 100%;
  border-collapse: collapse;
}

.ddns-detail-table th,
.ddns-detail-table td {
  padding: 8px 10px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.18);
  text-align: left;
  vertical-align: top;
}

.ddns-detail-table th {
  font-weight: 600;
}
</style>
