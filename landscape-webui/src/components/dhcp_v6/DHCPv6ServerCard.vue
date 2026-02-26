<script setup lang="ts">
import type { IPV6RAServiceConfig } from "@landscape-router/types/api/schemas";

const config = defineModel<IPV6RAServiceConfig>("service-config", {
  required: true,
});

function initialize_dhcpv6_if_needed() {
  if (!config.value?.config?.dhcpv6) {
    if (config.value?.config) {
      config.value.config.dhcpv6 = {
        enable: false,
        dns_servers: [],
      };
    }
  }
}

function initialize_ia_na(enable: boolean) {
  if (!config.value?.config) return;

  if (!config.value.config.dhcpv6) {
    config.value.config.dhcpv6 = {
      enable: false,
      dns_servers: [],
    };
  }

  if (enable) {
    config.value.config.dhcpv6.ia_na = {
      max_prefix_len: 64,
      pool_start: 256,
      preferred_lifetime: 3600,
      valid_lifetime: 7200,
    };
  } else {
    config.value.config.dhcpv6.ia_na = undefined;
  }
}

function initialize_ia_pd(enable: boolean) {
  if (!config.value?.config) return;

  if (!config.value.config.dhcpv6) {
    config.value.config.dhcpv6 = {
      enable: false,
      dns_servers: [],
    };
  }

  if (enable) {
    config.value.config.dhcpv6.ia_pd = {
      max_source_prefix_len: 56,
      delegate_prefix_len: 64,
      pool_start_index: 1,
      preferred_lifetime: 3600,
      valid_lifetime: 7200,
    };
  } else {
    config.value.config.dhcpv6.ia_pd = undefined;
  }
}

function update_ia_na_field(field: string, value: number | null) {
  if (!config.value?.config) return;

  initialize_dhcpv6_if_needed();

  const dhcpv6 = config.value.config.dhcpv6;
  if (!dhcpv6) return;

  if (!dhcpv6.ia_na) {
    dhcpv6.ia_na = {
      max_prefix_len: 64,
      pool_start: 256,
      preferred_lifetime: 3600,
      valid_lifetime: 7200,
    };
  }

  if (typeof value === "number") {
    (dhcpv6.ia_na as any)[field] = value;
  }
}

function update_ia_pd_field(field: string, value: number | null) {
  if (!config.value?.config) return;

  initialize_dhcpv6_if_needed();

  const dhcpv6 = config.value.config.dhcpv6;
  if (!dhcpv6) return;

  if (!dhcpv6.ia_pd) {
    dhcpv6.ia_pd = {
      max_source_prefix_len: 56,
      delegate_prefix_len: 64,
      pool_start_index: 1,
      preferred_lifetime: 3600,
      valid_lifetime: 7200,
    };
  }

  if (typeof value === "number") {
    (dhcpv6.ia_pd as any)[field] = value;
  }
}
</script>

<template>
  <n-card style="flex: 3; min-width: 0" size="small" :bordered="false">
    <template #header>
      <div style="display: flex; align-items: center; gap: 12px; flex: 1">
        <span>DHCPv6 服务器</span>
        <n-switch
          style="margin-left: auto"
          :value="!!config?.config.dhcpv6?.enable"
          @update:value="
            (val: boolean) => {
              initialize_dhcpv6_if_needed();
              if (config?.config?.dhcpv6) {
                config.config.dhcpv6.enable = val;
              }
            }
          "
        >
          <template #checked> 启用 </template>
          <template #unchecked> 禁用 </template>
        </n-switch>
      </div>
    </template>

    <!-- 内部改为两列 flex -->
    <n-flex :gap="12" align="start">
      <!-- 左列：IA_NA -->
      <div style="flex: 1; min-width: 0">
        <n-divider title-placement="left" style="margin: 0 0 8px"
          >IA_NA (地址分配)</n-divider
        >
        <n-grid :x-gap="12" :y-gap="8" cols="2" item-responsive>
          <n-form-item-gi span="2" label="启用 IA_NA">
            <n-switch
              :value="!!config?.config.dhcpv6?.ia_na"
              @update:value="initialize_ia_na"
            />
          </n-form-item-gi>
          <template v-if="config?.config.dhcpv6?.ia_na">
            <n-form-item-gi span="2">
              <template #label>
                <span style="display: flex; align-items: center; gap: 4px">
                  过滤前缀长度
                  <n-tooltip placement="top">
                    <template #trigger>
                      <span style="cursor: help; font-weight: bold">?</span>
                    </template>
                    限制分配给客户端的最大前缀长度
                  </n-tooltip>
                </span>
              </template>
              <n-input-number
                style="flex: 1"
                :value="config?.config.dhcpv6?.ia_na?.max_prefix_len ?? 64"
                @update:value="
                  (val: number | null) =>
                    update_ia_na_field('max_prefix_len', val)
                "
                :min="1"
                :max="127"
              />
            </n-form-item-gi>
            <n-form-item-gi span="2" label="地址池起始">
              <n-input-number
                style="flex: 1"
                :value="config?.config.dhcpv6?.ia_na?.pool_start ?? 256"
                @update:value="
                  (val: number | null) => update_ia_na_field('pool_start', val)
                "
                :min="1"
              />
            </n-form-item-gi>
            <n-form-item-gi span="1" label="首选生存期">
              <n-input-number
                style="flex: 1"
                :value="
                  config?.config.dhcpv6?.ia_na?.preferred_lifetime ?? 3600
                "
                @update:value="
                  (val: number | null) =>
                    update_ia_na_field('preferred_lifetime', val)
                "
                :min="1"
              />
            </n-form-item-gi>
            <n-form-item-gi span="1" label="有效生存期">
              <n-input-number
                style="flex: 1"
                :value="config?.config.dhcpv6?.ia_na?.valid_lifetime ?? 7200"
                @update:value="
                  (val: number | null) =>
                    update_ia_na_field('valid_lifetime', val)
                "
                :min="1"
              />
            </n-form-item-gi>
          </template>
        </n-grid>
      </div>

      <!-- 右列：IA_PD -->
      <div style="flex: 1; min-width: 0">
        <n-divider title-placement="left" style="margin: 0 0 8px"
          >IA_PD (前缀委派)</n-divider
        >
        <n-grid :x-gap="12" :y-gap="8" cols="2" item-responsive>
          <n-form-item-gi span="2" label="启用 IA_PD">
            <n-switch
              :value="!!config?.config.dhcpv6?.ia_pd"
              @update:value="initialize_ia_pd"
            />
          </n-form-item-gi>
          <template v-if="config?.config.dhcpv6?.ia_pd">
            <n-form-item-gi span="2">
              <template #label>
                <span style="display: flex; align-items: center; gap: 4px">
                  过滤源前缀长度
                  <n-tooltip placement="top">
                    <template #trigger>
                      <span style="cursor: help; font-weight: bold">?</span>
                    </template>
                    限制委派给客户端的最大源前缀长度
                  </n-tooltip>
                </span>
              </template>
              <n-input-number
                style="flex: 1"
                :value="
                  config?.config.dhcpv6?.ia_pd?.max_source_prefix_len ?? 56
                "
                @update:value="
                  (val: number | null) =>
                    update_ia_pd_field('max_source_prefix_len', val)
                "
                :min="1"
                :max="126"
              />
            </n-form-item-gi>
            <n-form-item-gi span="2" label="委派前缀长度">
              <n-input-number
                style="flex: 1"
                :value="config?.config.dhcpv6?.ia_pd?.delegate_prefix_len ?? 64"
                @update:value="
                  (val: number | null) =>
                    update_ia_pd_field('delegate_prefix_len', val)
                "
                :min="
                  (config?.config.dhcpv6?.ia_pd?.max_source_prefix_len ?? 56) +
                  1
                "
                :max="128"
              />
            </n-form-item-gi>
            <n-form-item-gi span="1" label="子前缀池起始">
              <n-input-number
                style="flex: 1"
                :value="config?.config.dhcpv6?.ia_pd?.pool_start_index ?? 1"
                @update:value="
                  (val: number | null) =>
                    update_ia_pd_field('pool_start_index', val)
                "
                :min="0"
              />
            </n-form-item-gi>
            <n-form-item-gi span="1" label="首选生存期">
              <n-input-number
                style="flex: 1"
                :value="
                  config?.config.dhcpv6?.ia_pd?.preferred_lifetime ?? 3600
                "
                @update:value="
                  (val: number | null) =>
                    update_ia_pd_field('preferred_lifetime', val)
                "
                :min="1"
              />
            </n-form-item-gi>
          </template>
        </n-grid>
      </div>
    </n-flex>
  </n-card>
</template>
