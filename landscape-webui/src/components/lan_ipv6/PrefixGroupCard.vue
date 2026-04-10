<script setup lang="ts">
import type { IPv6ServiceMode, LanPrefixGroupConfig } from "@landscape-router/types/api/schemas";
import { computed, ref } from "vue";
import { Edit } from "@vicons/carbon";
import { useI18n } from "vue-i18n";
import PrefixGroupEditorModal from "@/components/lan_ipv6/PrefixGroupEditorModal.vue";
import {
  groupParentLabel,
  sourceTypeFromParent,
} from "@/lib/lan_ipv6_v2_helpers";

const { t } = useI18n({ useScope: "global" });

type ServiceKind = "ra" | "na" | "pd";
type SourceType = "static" | "pd";

const props = defineProps<{
  group: LanPrefixGroupConfig;
  allowedServiceKinds: ServiceKind[];
  ifaceName: string;
  currentGroups: LanPrefixGroupConfig[];
  currentMode?: IPv6ServiceMode;
}>();

const emit = defineEmits<{
  (e: "commitGroup", groupKey: string, group: LanPrefixGroupConfig | undefined): void;
}>();

const showEdit = ref(false);
const initialKind = ref<ServiceKind>("ra");
const kinds: ServiceKind[] = ["ra", "na", "pd"];

const sourceType = computed(() => sourceTypeFromParent(props.group.parent));
const parentLabel = computed(() => groupParentLabel(props.group.parent));
const editorKinds = computed<ServiceKind[]>(() => {
  const set = new Set<ServiceKind>(props.allowedServiceKinds);
  if (props.group.ra) {
    set.add("ra");
  }
  if (props.group.na) {
    set.add("na");
  }
  if (props.group.pd) {
    set.add("pd");
  }
  return kinds.filter((kind) => set.has(kind));
});

function brushLabel(kind: ServiceKind) {
  return t(`lan_ipv6.planner_brush_${kind}`);
}

function canOpenKind(kind: ServiceKind) {
  return editorKinds.value.includes(kind) || kindConfigured(kind);
}

function kindSummary(kind: ServiceKind) {
  if (kind !== "pd" || !props.group.pd) {
    return undefined;
  }
  return t("lan_ipv6.prefix_pd_range", {
    prefix: `/${props.group.pd.pool_len}`,
    start: props.group.pd.start_index,
    end: props.group.pd.end_index,
  });
}

function kindConfigured(kind: ServiceKind) {
  if (kind === "ra") {
    return !!props.group.ra;
  }
  if (kind === "na") {
    return !!props.group.na;
  }
  return !!props.group.pd;
}

function kindDetail(kind: ServiceKind) {
  if (kind === "ra" && props.group.ra) {
    return `PL ${props.group.ra.preferred_lifetime}s · VL ${props.group.ra.valid_lifetime}s`;
  }
  return undefined;
}

function onCommit(group: LanPrefixGroupConfig | undefined) {
  emit("commitGroup", props.group.group_id, group);
}

function openEditor(kind: ServiceKind) {
  if (!canOpenKind(kind)) {
    return;
  }
  initialKind.value = kind;
  showEdit.value = true;
}

const rootClass = computed(() =>
  sourceType.value === "static" ? "group-card static-card" : "group-card pd-card",
);
</script>

<template>
  <n-card size="small" :bordered="false" :class="rootClass">
    <div class="group-summary compact">
      <div class="summary-parent">
        <div class="summary-parent-main">
          <n-tag size="small" :bordered="false" type="info">
            {{ parentLabel }}
          </n-tag>
        </div>
      </div>

      <div
        v-for="kind in kinds"
        :key="`${group.group_id}-${kind}`"
        class="summary-row"
        :class="{ clickable: canOpenKind(kind), configured: kindConfigured(kind), empty: !kindConfigured(kind) }"
        @click="canOpenKind(kind) && openEditor(kind)"
      >
        <div class="summary-main">
          <div class="summary-head">
            <div class="summary-kind">{{ brushLabel(kind) }}</div>
            <span class="summary-state" :class="kindConfigured(kind) ? 'configured' : 'empty'">
              {{
                kindConfigured(kind)
                  ? t('lan_ipv6.prefix_state_compact_configured')
                  : t('lan_ipv6.prefix_state_compact_empty')
              }}
            </span>
          </div>
          <div v-if="kindConfigured(kind) && kindSummary(kind)" class="summary-text">
            {{
              kindSummary(kind)
            }}
          </div>
          <div v-if="kindDetail(kind)" class="summary-detail">{{ kindDetail(kind) }}</div>
        </div>
      </div>

      <div class="summary-actions">
        <n-button
          size="small"
          secondary
          type="primary"
          @click="openEditor(editorKinds[0] ?? kinds[0])"
        >
          <template #icon>
            <n-icon><Edit /></n-icon>
          </template>
          {{ t('lan_ipv6.prefix_group_edit') }}
        </n-button>
      </div>
    </div>

    <PrefixGroupEditorModal
      v-model:show="showEdit"
      :source-type="sourceType"
      :parent-label="parentLabel"
      :group="group"
      :allowed-service-kinds="editorKinds"
      :current-iface-name="ifaceName"
      :current-groups="currentGroups"
      :current-mode="currentMode"
      :initial-kind="initialKind"
      @commit="onCommit"
    />
  </n-card>
</template>

<style scoped>
.group-card {
  margin-bottom: 8px;
}

.static-card {
  background: color-mix(in srgb, var(--n-color) 82%, var(--n-primary-color-suppl) 18%);
}

.pd-card {
  background: color-mix(in srgb, var(--n-color) 72%, var(--n-info-color-suppl) 28%);
}

.group-summary {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.group-summary.compact {
  margin-top: 0;
  display: grid;
  grid-template-columns: minmax(220px, 1.2fr) repeat(3, minmax(0, 1fr)) auto;
  gap: 8px;
  align-items: stretch;
}

.summary-parent {
  border: 1px solid var(--n-border-color);
  border-radius: 10px;
  background: color-mix(in srgb, var(--n-color) 88%, var(--n-hover-color) 12%);
  padding: 10px 12px;
  min-width: 0;
}

.summary-parent-main {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.summary-parent-hint {
  font-size: 12px;
  color: var(--n-text-color-3);
  margin-top: 6px;
}

.summary-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid var(--n-border-color);
  border-radius: 10px;
  background: color-mix(in srgb, var(--n-color) 90%, var(--n-color-embedded) 10%);
  min-width: 0;
}

.summary-row.configured {
  background: color-mix(in srgb, var(--n-color) 84%, var(--n-hover-color) 16%);
}

.summary-row.empty {
  opacity: 0.88;
}

.summary-row.clickable {
  cursor: pointer;
}

.summary-row.clickable:hover {
  background: color-mix(in srgb, var(--n-color) 78%, var(--n-hover-color) 22%);
}

.summary-main {
  min-width: 0;
  flex: 1;
}

.summary-head {
  display: flex;
  align-items: center;
  gap: 8px;
}

.summary-kind {
  font-weight: 600;
  font-size: 13px;
}

.summary-state {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  padding: 1px 8px;
  font-size: 11px;
  line-height: 18px;
}

.summary-state.configured {
  background: color-mix(in srgb, var(--n-success-color) 16%, transparent);
  color: var(--n-success-color);
}

.summary-state.empty {
  background: color-mix(in srgb, var(--n-text-color-disabled) 14%, transparent);
  color: var(--n-text-color-3);
}

.summary-text {
  font-size: 13px;
  margin-top: 2px;
}

.summary-detail {
  font-size: 12px;
  color: var(--n-text-color-3);
  margin-top: 2px;
}

.summary-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

@media (max-width: 900px) {
  .group-summary.compact {
    grid-template-columns: 1fr;
  }
}
</style>
