<script setup lang="ts">
import { ref, watch } from "vue";
import { FormInst, useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";
import { ZoneType } from "@/lib/service_ipconfig";
import { useIPv6PDStore } from "@/stores/status_ipv6pd";
import {
  get_lan_ipv6_config,
  update_lan_ipv6_config,
} from "@/api/service_lan_ipv6";
import type {
  IPV6RaConfigSource,
  IPv6RaPdConfig,
  LanIPv6ServiceConfig,
  IPv6RaStaticConfig,
  IPv6ServiceMode,
} from "@landscape-router/types/api/schemas";
import DHCPv6ConfigSection from "@/components/dhcp_v6/DHCPv6ConfigSection.vue";
import DHCPv6ServerCard from "@/components/dhcp_v6/DHCPv6ServerCard.vue";

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
      source: [],
      dhcpv6: {
        enable: false,
        source: [],
      },
    },
  };
}

async function on_modal_enter() {
  try {
    let config = await get_lan_ipv6_config(iface_info.iface_name);
    if (config) {
      service_config.value = config;
    } else {
      service_config.value = default_config();
    }
    // Always ensure dhcpv6 config is initialized
    if (!service_config.value.config.dhcpv6) {
      service_config.value.config.dhcpv6 = {
        enable: false,
        source: [],
      };
    }
    if (!service_config.value.config.dhcpv6.source) {
      service_config.value.config.dhcpv6.source = [];
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
        source: [],
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
        source: [],
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
      if (!validate_ra_source(service_config.value.config.source)) {
        return;
      }
      // For stateful and slaac_dhcpv6 modes, validate DHCPv6 source
      const mode = service_config.value.config.mode;
      if (mode === "stateful" || mode === "slaac_dhcpv6") {
        const dhcpv6_source = service_config.value.config.dhcpv6?.source ?? [];
        if (!validate_ra_source(dhcpv6_source)) {
          return;
        }
        // Cross-source sub_index uniqueness for slaac_dhcpv6
        if (mode === "slaac_dhcpv6") {
          if (
            !validate_cross_source_uniqueness(
              service_config.value.config.source,
              dhcpv6_source,
            )
          ) {
            return;
          }
        }
      }
      await update_lan_ipv6_config(service_config.value);
      await ipv6PDStore.UPDATE_INFO();
      show_model.value = false;
    }
  } catch (err) {
    message.warning(t("lan_ipv6.form_validation_failed"));
  }
}

const formRules = {};

// RA source edit
const show_source_edit = ref(false);
function add_source(source: IPV6RaConfigSource) {
  service_config.value?.config.source.unshift(source);
}

function replace_source(source: IPV6RaConfigSource, index: number) {
  if (service_config.value) {
    service_config.value.config.source[index] = source;
  }
}

function delete_source(index: number) {
  if (service_config.value) {
    service_config.value.config.source.splice(index, 1);
  }
}

// DHCPv6 source edit
const show_dhcpv6_source_edit = ref(false);
function add_dhcpv6_source(source: IPV6RaConfigSource) {
  if (service_config.value?.config.dhcpv6) {
    if (!service_config.value.config.dhcpv6.source) {
      service_config.value.config.dhcpv6.source = [];
    }
    service_config.value.config.dhcpv6.source.unshift(source);
  }
}

function replace_dhcpv6_source(source: IPV6RaConfigSource, index: number) {
  if (service_config.value?.config.dhcpv6?.source) {
    service_config.value.config.dhcpv6.source[index] = source;
  }
}

function delete_dhcpv6_source(index: number) {
  if (service_config.value?.config.dhcpv6?.source) {
    service_config.value.config.dhcpv6.source.splice(index, 1);
  }
}

function validate_ra_source(source: IPV6RaConfigSource[]): boolean {
  const basePrefixes = new Set<string>();
  const dependIfaces = new Set<string>();
  const subnetIndices = new Set<number>();

  for (const src of source) {
    switch (src.t) {
      case "static": {
        const s = src as IPv6RaStaticConfig;
        if (basePrefixes.has(s.base_prefix)) {
          window.$message.warning(`重复的静态前缀配置: ${s.base_prefix}`);
          return false;
        }
        basePrefixes.add(s.base_prefix);

        if (subnetIndices.has(s.sub_index)) {
          window.$message.warning(`重复的子网索引: ${s.sub_index}`);
          return false;
        }
        subnetIndices.add(s.sub_index);
        break;
      }
      case "pd": {
        const p = src as IPv6RaPdConfig;
        if (dependIfaces.has(p.depend_iface)) {
          window.$message.warning(`重复的网卡: ${p.depend_iface}`);
          return false;
        }
        dependIfaces.add(p.depend_iface);

        if (subnetIndices.has(p.subnet_index)) {
          window.$message.warning(`重复的子网索引: ${p.subnet_index}`);
          return false;
        }
        subnetIndices.add(p.subnet_index);
        break;
      }
    }
  }

  return true;
}

function validate_cross_source_uniqueness(
  ra_source: IPV6RaConfigSource[],
  dhcpv6_source: IPV6RaConfigSource[],
): boolean {
  const subnetIndices = new Set<number>();

  for (const src of ra_source) {
    if (src.t === "static") {
      subnetIndices.add((src as IPv6RaStaticConfig).sub_index);
    } else if (src.t === "pd") {
      subnetIndices.add((src as IPv6RaPdConfig).subnet_index);
    }
  }

  for (const src of dhcpv6_source) {
    let idx: number;
    if (src.t === "static") {
      idx = (src as IPv6RaStaticConfig).sub_index;
    } else {
      idx = (src as IPv6RaPdConfig).subnet_index;
    }
    if (subnetIndices.has(idx)) {
      window.$message.warning(t("lan_ipv6.cross_source_conflict", { idx }));
      return false;
    }
  }

  return true;
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
              @click="show_source_edit = true"
            >
              {{ t("lan_ipv6.add") }}
            </n-button>
            <ICMPRaSourceEdit
              @commit="add_source"
              v-model:show="show_source_edit"
            ></ICMPRaSourceEdit>
          </template>

          <n-text
            depth="3"
            style="font-size: 12px; display: block; margin-bottom: 8px"
          >
            {{ t("lan_ipv6.ra_prefix_source_desc") }}
          </n-text>

          <n-scrollbar style="max-height: 300px">
            <n-flex v-if="service_config.config.source.length > 0">
              <ICMPRaSourceExhibit
                v-for="(each, index) in service_config.config.source"
                :source="each"
                @commit="(e: any) => replace_source(e, index)"
                @delete="delete_source(index)"
              >
              </ICMPRaSourceExhibit>
            </n-flex>
            <n-empty v-else :description="t('lan_ipv6.no_prefix')" />
          </n-scrollbar>
        </n-card>

        <!-- Mode 2 (Stateful): DHCPv6 prefix source full-width -->
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
              @click="show_dhcpv6_source_edit = true"
            >
              {{ t("lan_ipv6.add") }}
            </n-button>
            <ICMPRaSourceEdit
              @commit="add_dhcpv6_source"
              v-model:show="show_dhcpv6_source_edit"
            ></ICMPRaSourceEdit>
          </template>

          <n-text
            depth="3"
            style="font-size: 12px; display: block; margin-bottom: 8px"
          >
            {{ t("lan_ipv6.dhcpv6_prefix_source_desc") }}
          </n-text>

          <n-scrollbar style="max-height: 300px">
            <n-flex
              v-if="(service_config.config.dhcpv6?.source ?? []).length > 0"
            >
              <ICMPRaSourceExhibit
                v-for="(each, index) in service_config.config.dhcpv6?.source ??
                []"
                :source="each"
                @commit="(e: any) => replace_dhcpv6_source(e, index)"
                @delete="delete_dhcpv6_source(index)"
              >
              </ICMPRaSourceExhibit>
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
                @click="show_source_edit = true"
              >
                {{ t("lan_ipv6.add") }}
              </n-button>
              <ICMPRaSourceEdit
                @commit="add_source"
                v-model:show="show_source_edit"
              ></ICMPRaSourceEdit>
            </template>

            <n-text
              depth="3"
              style="font-size: 12px; display: block; margin-bottom: 8px"
            >
              {{ t("lan_ipv6.ra_prefix_source_ula_desc") }}
            </n-text>

            <n-scrollbar style="max-height: 300px">
              <n-flex v-if="service_config.config.source.length > 0">
                <ICMPRaSourceExhibit
                  v-for="(each, index) in service_config.config.source"
                  :source="each"
                  @commit="(e: any) => replace_source(e, index)"
                  @delete="delete_source(index)"
                >
                </ICMPRaSourceExhibit>
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
                @click="show_dhcpv6_source_edit = true"
              >
                {{ t("lan_ipv6.add") }}
              </n-button>
              <ICMPRaSourceEdit
                @commit="add_dhcpv6_source"
                v-model:show="show_dhcpv6_source_edit"
              ></ICMPRaSourceEdit>
            </template>

            <n-text
              depth="3"
              style="font-size: 12px; display: block; margin-bottom: 8px"
            >
              {{ t("lan_ipv6.dhcpv6_prefix_source_combo_desc") }}
            </n-text>

            <n-scrollbar style="max-height: 300px">
              <n-flex
                v-if="(service_config.config.dhcpv6?.source ?? []).length > 0"
              >
                <ICMPRaSourceExhibit
                  v-for="(each, index) in service_config.config.dhcpv6
                    ?.source ?? []"
                  :source="each"
                  @commit="(e: any) => replace_dhcpv6_source(e, index)"
                  @delete="delete_dhcpv6_source(index)"
                >
                </ICMPRaSourceExhibit>
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
