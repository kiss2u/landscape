<script setup lang="ts">
import { ref } from "vue";
import { FormInst, useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";
import { ZoneType } from "@/lib/service_ipconfig";
import { useIPv6PDStore } from "@/stores/status_ipv6pd";
import {
  get_lan_ipv6_config,
  update_lan_ipv6_config,
} from "@/api/service_lan_ipv6";
import type {
  LanIPv6ServiceConfig,
  LanIPv6SourceConfig,
  IPv6ServiceMode,
} from "@landscape-router/types/api/schemas";
import DHCPv6ServerCard from "@/components/dhcp_v6/DHCPv6ServerCard.vue";
import SourceBindingCard from "@/components/lan_ipv6/SourceBindingCard.vue";
import SourceBindingModal from "@/components/lan_ipv6/SourceBindingModal.vue";

const { t } = useI18n({ useScope: "global" });
let ipv6PDStore = useIPv6PDStore();
const message = useMessage();

const show_model = defineModel<boolean>("show", { required: true });
const emit = defineEmits(["refresh"]);
const formRef = ref<FormInst | null>(null);

const iface_info = defineProps<{
  iface_name: string;
  mac?: string;
  zone: ZoneType;
}>();

const service_config = ref<LanIPv6ServiceConfig>();

function default_config(): LanIPv6ServiceConfig {
  return {
    iface_name: iface_info.iface_name,
    enable: true,
    config: {
      mode: "slaac" as IPv6ServiceMode,
      ad_interval: 300,
      ra_flag: {
        managed_address_config: false,
        other_config: false,
        home_agent: false,
        prf: 0,
        nd_proxy: false,
        reserved: 0,
      },
      sources: [],
      dhcpv6: {
        enable: false,
      },
    },
  };
}

// Filter sources by service kind
function sources_by_kind(kind: "ra" | "na" | "pd"): LanIPv6SourceConfig[] {
  if (!service_config.value) return [];
  const sources = service_config.value.config.sources ?? [];
  switch (kind) {
    case "ra":
      return sources.filter((s) => s.t === "ra_static" || s.t === "ra_pd");
    case "na":
      return sources.filter((s) => s.t === "na_static" || s.t === "na_pd");
    case "pd":
      return sources.filter((s) => s.t === "pd_static" || s.t === "pd_pd");
  }
}

// Find index in the flat sources array for a filtered item
function find_source_index(
  filtered_index: number,
  kind: "ra" | "na" | "pd",
): number {
  if (!service_config.value) return -1;
  const sources = service_config.value.config.sources ?? [];
  let count = 0;
  for (let i = 0; i < sources.length; i++) {
    const s = sources[i];
    const matches =
      (kind === "ra" && (s.t === "ra_static" || s.t === "ra_pd")) ||
      (kind === "na" && (s.t === "na_static" || s.t === "na_pd")) ||
      (kind === "pd" && (s.t === "pd_static" || s.t === "pd_pd"));
    if (matches) {
      if (count === filtered_index) return i;
      count++;
    }
  }
  return -1;
}

async function on_modal_enter() {
  try {
    let config = await get_lan_ipv6_config(iface_info.iface_name);
    if (config) {
      service_config.value = config;
    } else {
      service_config.value = default_config();
    }
    // Ensure sources is initialized
    if (!service_config.value.config.sources) {
      service_config.value.config.sources = [];
    }
    // Always ensure dhcpv6 config is initialized
    if (!service_config.value.config.dhcpv6) {
      service_config.value.config.dhcpv6 = {
        enable: false,
      };
    }
    // Default mode to slaac if not set
    if (!service_config.value.config.mode) {
      service_config.value.config.mode = "slaac" as IPv6ServiceMode;
    }
  } catch (e) {
    service_config.value = default_config();
  }
}

function on_mode_change(mode: IPv6ServiceMode) {
  if (!service_config.value) return;
  service_config.value.config.mode = mode;
  // Auto-set flags based on mode
  if (mode === "slaac") {
    service_config.value.config.ra_flag.managed_address_config = false;
    service_config.value.config.ra_flag.other_config = false;
    // Disable DHCPv6
    if (service_config.value.config.dhcpv6) {
      service_config.value.config.dhcpv6.enable = false;
    }
  } else if (mode === "stateful") {
    service_config.value.config.ra_flag.managed_address_config = true;
    service_config.value.config.ra_flag.other_config = true;
    // Enable DHCPv6
    if (!service_config.value.config.dhcpv6) {
      service_config.value.config.dhcpv6 = {
        enable: true,
      };
    } else {
      service_config.value.config.dhcpv6.enable = true;
    }
  } else if (mode === "slaac_dhcpv6") {
    service_config.value.config.ra_flag.managed_address_config = true;
    service_config.value.config.ra_flag.other_config = true;
    // Enable DHCPv6
    if (!service_config.value.config.dhcpv6) {
      service_config.value.config.dhcpv6 = {
        enable: true,
      };
    } else {
      service_config.value.config.dhcpv6.enable = true;
    }
  }
}

async function save_config() {
  try {
    await formRef.value?.validate();
    if (service_config.value) {
      await update_lan_ipv6_config(service_config.value);
      await ipv6PDStore.UPDATE_INFO();
      show_model.value = false;
    }
  } catch (err) {
    message.warning(t("lan_ipv6.form_validation_failed"));
  }
}

const formRules = {};

// Source edit modals
const show_ra_source_edit = ref(false);
const show_na_source_edit = ref(false);
const show_pd_source_edit = ref(false);

function add_source(source: LanIPv6SourceConfig) {
  if (service_config.value) {
    if (!service_config.value.config.sources) {
      service_config.value.config.sources = [];
    }
    service_config.value.config.sources.unshift(source);
  }
}

function replace_source(source: LanIPv6SourceConfig, flat_index: number) {
  if (service_config.value?.config.sources) {
    service_config.value.config.sources[flat_index] = source;
  }
}

function delete_source(flat_index: number) {
  if (service_config.value?.config.sources) {
    service_config.value.config.sources.splice(flat_index, 1);
  }
}
</script>

<template>
  <n-modal
    :auto-focus="false"
    v-model:show="show_model"
    @after-enter="on_modal_enter"
  >
    <n-card
      style="width: 1200px"
      :title="t('lan_ipv6.title')"
      :bordered="false"
      size="small"
      role="dialog"
      aria-modal="true"
      closable
      @close="show_model = false"
    >
      <n-form
        v-if="service_config"
        ref="formRef"
        :model="service_config"
        :rules="formRules"
      >
        <!-- Mode selector -->
        <n-card
          style="width: 100%; margin-bottom: 12px"
          size="small"
          :bordered="false"
        >
          <n-flex align="center" :gap="16">
            <n-form-item :label="t('lan_ipv6.enable')" style="margin-bottom: 0">
              <n-switch v-model:value="service_config.enable">
                <template #checked> {{ t("lan_ipv6.enabled") }} </template>
                <template #unchecked> {{ t("lan_ipv6.disabled") }} </template>
              </n-switch>
            </n-form-item>
            <n-form-item
              :label="t('lan_ipv6.mode')"
              style="margin-bottom: 0; flex: 1"
            >
              <n-radio-group
                :value="service_config.config.mode"
                @update:value="on_mode_change"
                name="ipv6-mode"
              >
                <n-radio-button
                  value="slaac"
                  :label="t('lan_ipv6.mode_slaac')"
                />
                <n-radio-button
                  value="stateful"
                  :label="t('lan_ipv6.mode_stateful')"
                />
                <n-radio-button
                  value="slaac_dhcpv6"
                  :label="t('lan_ipv6.mode_slaac_dhcpv6')"
                />
              </n-radio-group>
            </n-form-item>
          </n-flex>

          <n-alert
            v-if="service_config.config.mode === 'slaac'"
            type="info"
            :bordered="false"
            style="margin-top: 8px"
          >
            {{ t("lan_ipv6.mode_slaac_desc") }}
          </n-alert>
          <n-alert
            v-else-if="service_config.config.mode === 'stateful'"
            type="info"
            :bordered="false"
            style="margin-top: 8px"
          >
            {{ t("lan_ipv6.mode_stateful_desc") }}
          </n-alert>
          <n-alert
            v-else-if="service_config.config.mode === 'slaac_dhcpv6'"
            type="info"
            :bordered="false"
            style="margin-top: 8px"
          >
            {{ t("lan_ipv6.mode_slaac_dhcpv6_desc") }}
          </n-alert>
        </n-card>

        <!-- Mode 1 (Slaac): RA prefix source full-width -->
        <n-card
          v-if="service_config.config.mode === 'slaac'"
          style="width: 100%; margin-bottom: 12px"
          size="small"
          :title="t('lan_ipv6.ra_prefix_source')"
          :bordered="false"
        >
          <template #header-extra>
            <button
              style="
                width: 0;
                height: 0;
                overflow: hidden;
                opacity: 0;
                position: absolute;
              "
            ></button>
            <n-button
              :focusable="false"
              size="tiny"
              @click="show_ra_source_edit = true"
            >
              {{ t("lan_ipv6.add") }}
            </n-button>
            <SourceBindingModal
              @commit="add_source"
              v-model:show="show_ra_source_edit"
              :allowed-service-kinds="['ra']"
            />
          </template>

          <n-text
            depth="3"
            style="font-size: 12px; display: block; margin-bottom: 8px"
          >
            {{ t("lan_ipv6.ra_prefix_source_desc") }}
          </n-text>

          <n-scrollbar style="max-height: 300px">
            <n-flex v-if="sources_by_kind('ra').length > 0">
              <SourceBindingCard
                v-for="(each, index) in sources_by_kind('ra')"
                :key="index"
                :source="each"
                :allowed-service-kinds="['ra']"
                @commit="
                  (e: any) => replace_source(e, find_source_index(index, 'ra'))
                "
                @delete="delete_source(find_source_index(index, 'ra'))"
              />
            </n-flex>
            <n-empty v-else :description="t('lan_ipv6.no_prefix')" />
          </n-scrollbar>
        </n-card>

        <!-- Mode 2 (Stateful): DHCPv6 prefix sources (NA + PD) full-width -->
        <n-card
          v-if="service_config.config.mode === 'stateful'"
          style="width: 100%; margin-bottom: 12px"
          size="small"
          :title="t('lan_ipv6.dhcpv6_prefix_source')"
          :bordered="false"
        >
          <template #header-extra>
            <button
              style="
                width: 0;
                height: 0;
                overflow: hidden;
                opacity: 0;
                position: absolute;
              "
            ></button>
            <n-button
              :focusable="false"
              size="tiny"
              @click="show_na_source_edit = true"
            >
              {{ t("lan_ipv6.add") }}
            </n-button>
            <SourceBindingModal
              @commit="add_source"
              v-model:show="show_na_source_edit"
              :allowed-service-kinds="['na', 'pd']"
            />
          </template>

          <n-text
            depth="3"
            style="font-size: 12px; display: block; margin-bottom: 8px"
          >
            {{ t("lan_ipv6.dhcpv6_prefix_source_desc") }}
          </n-text>

          <n-scrollbar style="max-height: 300px">
            <n-flex
              v-if="
                [...sources_by_kind('na'), ...sources_by_kind('pd')].length > 0
              "
            >
              <template
                v-for="(each, index) in sources_by_kind('na')"
                :key="'na-' + index"
              >
                <SourceBindingCard
                  :source="each"
                  :allowed-service-kinds="['na', 'pd']"
                  @commit="
                    (e: any) =>
                      replace_source(e, find_source_index(index, 'na'))
                  "
                  @delete="delete_source(find_source_index(index, 'na'))"
                />
              </template>
              <template
                v-for="(each, index) in sources_by_kind('pd')"
                :key="'pd-' + index"
              >
                <SourceBindingCard
                  :source="each"
                  :allowed-service-kinds="['na', 'pd']"
                  @commit="
                    (e: any) =>
                      replace_source(e, find_source_index(index, 'pd'))
                  "
                  @delete="delete_source(find_source_index(index, 'pd'))"
                />
              </template>
            </n-flex>
            <n-empty v-else :description="t('lan_ipv6.no_dhcpv6_prefix')" />
          </n-scrollbar>
        </n-card>

        <!-- Mode 3 (SlaacDhcpv6): RA + DHCPv6 prefix sources side by side -->
        <n-flex
          v-if="service_config.config.mode === 'slaac_dhcpv6'"
          :gap="12"
          align="stretch"
          style="margin-bottom: 12px"
        >
          <!-- Left: RA prefix source (ULA static) -->
          <n-card
            style="flex: 1; min-width: 0"
            size="small"
            :title="t('lan_ipv6.ra_prefix_source_ula')"
            :bordered="false"
          >
            <template #header-extra>
              <button
                style="
                  width: 0;
                  height: 0;
                  overflow: hidden;
                  opacity: 0;
                  position: absolute;
                "
              ></button>
              <n-button
                :focusable="false"
                size="tiny"
                @click="show_ra_source_edit = true"
              >
                {{ t("lan_ipv6.add") }}
              </n-button>
              <SourceBindingModal
                @commit="add_source"
                v-model:show="show_ra_source_edit"
                :allowed-service-kinds="['ra']"
              />
            </template>

            <n-text
              depth="3"
              style="font-size: 12px; display: block; margin-bottom: 8px"
            >
              {{ t("lan_ipv6.ra_prefix_source_ula_desc") }}
            </n-text>

            <n-scrollbar style="max-height: 300px">
              <n-flex v-if="sources_by_kind('ra').length > 0">
                <SourceBindingCard
                  v-for="(each, index) in sources_by_kind('ra')"
                  :key="index"
                  :source="each"
                  :allowed-service-kinds="['ra']"
                  @commit="
                    (e: any) =>
                      replace_source(e, find_source_index(index, 'ra'))
                  "
                  @delete="delete_source(find_source_index(index, 'ra'))"
                />
              </n-flex>
              <n-empty v-else :description="t('lan_ipv6.no_ra_prefix')" />
            </n-scrollbar>
          </n-card>

          <!-- Right: DHCPv6 prefix source -->
          <n-card
            style="flex: 1; min-width: 0"
            size="small"
            :title="t('lan_ipv6.dhcpv6_prefix_source')"
            :bordered="false"
          >
            <template #header-extra>
              <button
                style="
                  width: 0;
                  height: 0;
                  overflow: hidden;
                  opacity: 0;
                  position: absolute;
                "
              ></button>
              <n-button
                :focusable="false"
                size="tiny"
                @click="show_na_source_edit = true"
              >
                {{ t("lan_ipv6.add") }}
              </n-button>
              <SourceBindingModal
                @commit="add_source"
                v-model:show="show_na_source_edit"
                :allowed-service-kinds="['na', 'pd']"
              />
            </template>

            <n-text
              depth="3"
              style="font-size: 12px; display: block; margin-bottom: 8px"
            >
              {{ t("lan_ipv6.dhcpv6_prefix_source_combo_desc") }}
            </n-text>

            <n-scrollbar style="max-height: 300px">
              <n-flex
                v-if="
                  [...sources_by_kind('na'), ...sources_by_kind('pd')].length >
                  0
                "
              >
                <template
                  v-for="(each, index) in sources_by_kind('na')"
                  :key="'na-' + index"
                >
                  <SourceBindingCard
                    :source="each"
                    :allowed-service-kinds="['na', 'pd']"
                    @commit="
                      (e: any) =>
                        replace_source(e, find_source_index(index, 'na'))
                    "
                    @delete="delete_source(find_source_index(index, 'na'))"
                  />
                </template>
                <template
                  v-for="(each, index) in sources_by_kind('pd')"
                  :key="'pd-' + index"
                >
                  <SourceBindingCard
                    :source="each"
                    :allowed-service-kinds="['na', 'pd']"
                    @commit="
                      (e: any) =>
                        replace_source(e, find_source_index(index, 'pd'))
                    "
                    @delete="delete_source(find_source_index(index, 'pd'))"
                  />
                </template>
              </n-flex>
              <n-empty v-else :description="t('lan_ipv6.no_dhcpv6_prefix')" />
            </n-scrollbar>
          </n-card>
        </n-flex>

        <!-- Bottom config area -->
        <n-flex :gap="12" align="stretch">
          <!-- RA config -->
          <n-card
            style="flex: 1; min-width: 0"
            size="small"
            :title="t('lan_ipv6.ra_config')"
            :bordered="false"
          >
            <n-grid :x-gap="12" :y-gap="8" cols="2" item-responsive>
              <n-form-item-gi span="2">
                <template #label>
                  <Notice>
                    {{ t("lan_ipv6.ad_interval") }}
                    <template #msg>
                      {{ t("lan_ipv6.ad_interval_desc") }}
                    </template>
                  </Notice>
                </template>
                <n-input-number
                  style="flex: 1"
                  v-model:value="service_config.config.ad_interval"
                  clearable
                />
              </n-form-item-gi>

              <!-- M/O flags: show read-only for stateful/slaac_dhcpv6, editable for slaac -->
              <template v-if="service_config.config.mode === 'slaac'">
                <n-form-item-gi span="2">
                  <template #label>
                    <Notice>
                      {{ t("lan_ipv6.m_flag") }}
                      <template #msg>
                        {{ t("lan_ipv6.m_flag_desc") }}
                      </template>
                    </Notice>
                  </template>
                  <n-switch
                    v-model:value="
                      service_config.config.ra_flag.managed_address_config
                    "
                  />
                </n-form-item-gi>
                <n-form-item-gi span="2">
                  <template #label>
                    <Notice>
                      {{ t("lan_ipv6.o_flag") }}
                      <template #msg>
                        {{ t("lan_ipv6.o_flag_desc") }}
                      </template>
                    </Notice>
                  </template>
                  <n-switch
                    v-model:value="service_config.config.ra_flag.other_config"
                  />
                </n-form-item-gi>
              </template>
              <template v-else>
                <n-form-item-gi span="2">
                  <template #label>
                    <Notice>
                      {{ t("lan_ipv6.ra_flags_auto") }}
                      <template #msg>
                        {{ t("lan_ipv6.ra_flags_auto_desc") }}
                      </template>
                    </Notice>
                  </template>
                  <n-tag :bordered="false" type="info"> M=1, O=1 </n-tag>
                </n-form-item-gi>
              </template>

              <n-form-item-gi span="2" :label="t('lan_ipv6.route_priority')">
                <n-radio-group
                  v-model:value="service_config.config.ra_flag.prf"
                  name="ra_flag"
                >
                  <n-radio-button
                    :value="3"
                    :label="t('lan_ipv6.priority_low')"
                  />
                  <n-radio-button
                    :value="0"
                    :label="t('lan_ipv6.priority_medium')"
                  />
                  <n-radio-button
                    :value="1"
                    :label="t('lan_ipv6.priority_high')"
                  />
                </n-radio-group>
              </n-form-item-gi>
            </n-grid>
          </n-card>

          <!-- DHCPv6 Server Config (only for stateful and slaac_dhcpv6) -->
          <DHCPv6ServerCard
            v-if="
              service_config.config.mode === 'stateful' ||
              service_config.config.mode === 'slaac_dhcpv6'
            "
            v-model:service-config="service_config"
          />
        </n-flex>
      </n-form>
      <template #footer>
        <n-flex justify="end">
          <n-button round type="primary" @click="save_config">
            {{ t("lan_ipv6.update") }}
          </n-button>
        </n-flex>
      </template>
    </n-card>
  </n-modal>
</template>
