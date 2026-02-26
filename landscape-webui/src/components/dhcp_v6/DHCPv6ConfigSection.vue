<script setup lang="ts">
import { computed } from "vue";
import type {
  DHCPv6ServerConfig,
  DHCPv6IANAConfig,
  DHCPv6IAPDConfig,
} from "@landscape-router/types/api/schemas";

const props = defineProps<{
  managed_address_config: boolean;
}>();

const config = defineModel<DHCPv6ServerConfig | undefined>("config", {
  required: true,
});

const enabled = computed({
  get: () => config.value?.enable ?? false,
  set: (val: boolean) => {
    if (!config.value) {
      config.value = {
        enable: val,
        dns_servers: [],
      };
    } else {
      config.value.enable = val;
    }
  },
});

const ia_na_enabled = computed({
  get: () => !!config.value?.ia_na,
  set: (val: boolean) => {
    if (!config.value) return;
    if (val) {
      config.value.ia_na = {
        max_prefix_len: 64,
        pool_start: 256,
        preferred_lifetime: 3600,
        valid_lifetime: 7200,
      };
    } else {
      config.value.ia_na = undefined;
    }
  },
});

const ia_pd_enabled = computed({
  get: () => !!config.value?.ia_pd,
  set: (val: boolean) => {
    if (!config.value) return;
    if (val) {
      config.value.ia_pd = {
        max_source_prefix_len: 56,
        delegate_prefix_len: 64,
        pool_start_index: 1,
        preferred_lifetime: 3600,
        valid_lifetime: 7200,
      };
    } else {
      config.value.ia_pd = undefined;
    }
  },
});

const dnsServersStr = computed({
  get: () => config.value?.dns_servers?.join(", ") ?? "",
  set: (val: string) => {
    if (!config.value) return;
    config.value.dns_servers = val
      .split(/[,\s]+/)
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
  },
});

const showMFlagWarning = computed(() => {
  return enabled.value && !props.managed_address_config;
});
</script>

<template>
  <n-grid :x-gap="12" :y-gap="8" cols="4" item-responsive>
    <n-form-item-gi span="2" label="启用 DHCPv6">
      <n-switch v-model:value="enabled">
        <template #checked> 启用 </template>
        <template #unchecked> 禁用 </template>
      </n-switch>
    </n-form-item-gi>

    <n-form-item-gi span="4" v-if="showMFlagWarning">
      <n-alert type="warning" :bordered="false">
        DHCPv6 已启用但 RA M 标志未设置，客户端可能不会请求 DHCPv6 地址
      </n-alert>
    </n-form-item-gi>

    <template v-if="enabled && config">
      <!-- IA_NA Section -->
      <n-form-item-gi span="4" label="IA_NA (地址分配)">
        <n-switch v-model:value="ia_na_enabled">
          <template #checked> 启用 </template>
          <template #unchecked> 禁用 </template>
        </n-switch>
      </n-form-item-gi>

      <template v-if="config.ia_na">
        <n-form-item-gi span="2">
          <template #label>
            <Notice>
              最大前缀长度
              <template #msg>
                RA 中前缀长度 &le; 此值的前缀将用于地址分配<br />
                例如: 64 表示 /64 及更短的前缀可用
              </template>
            </Notice>
          </template>
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_na.max_prefix_len"
            :min="1"
            :max="127"
          />
        </n-form-item-gi>

        <n-form-item-gi span="2" label="地址池起始后缀">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_na.pool_start"
            :min="1"
          />
        </n-form-item-gi>

        <n-form-item-gi span="2" label="地址池结束后缀">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_na.pool_end"
            :min="config.ia_na.pool_start + 1"
            placeholder="默认: 起始 + 65535"
            clearable
          />
        </n-form-item-gi>

        <n-form-item-gi span="1" label="首选生存期(秒)">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_na.preferred_lifetime"
            :min="1"
          />
        </n-form-item-gi>

        <n-form-item-gi span="1" label="有效生存期(秒)">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_na.valid_lifetime"
            :min="1"
          />
        </n-form-item-gi>
      </template>

      <!-- IA_PD Section -->
      <n-form-item-gi span="4" label="IA_PD (前缀委派)">
        <n-switch v-model:value="ia_pd_enabled">
          <template #checked> 启用 </template>
          <template #unchecked> 禁用 </template>
        </n-switch>
      </n-form-item-gi>

      <template v-if="config.ia_pd">
        <n-form-item-gi span="2">
          <template #label>
            <Notice>
              最大源前缀长度
              <template #msg>
                RA 中前缀长度 &le; 此值的前缀将用于委派<br />
                例如: 56 表示仅 /56 及更短的前缀可用
              </template>
            </Notice>
          </template>
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_pd.max_source_prefix_len"
            :min="1"
            :max="126"
          />
        </n-form-item-gi>

        <n-form-item-gi span="2" label="委派前缀长度">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_pd.delegate_prefix_len"
            :min="(config.ia_pd.max_source_prefix_len ?? 56) + 1"
            :max="128"
          />
        </n-form-item-gi>

        <n-form-item-gi span="2" label="子前缀池起始索引">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_pd.pool_start_index"
            :min="0"
          />
        </n-form-item-gi>

        <n-form-item-gi span="2" label="子前缀池结束索引">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_pd.pool_end_index"
            :min="config.ia_pd.pool_start_index + 1"
            placeholder="默认: 自动计算"
            clearable
          />
        </n-form-item-gi>

        <n-form-item-gi span="1" label="首选生存期(秒)">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_pd.preferred_lifetime"
            :min="1"
          />
        </n-form-item-gi>

        <n-form-item-gi span="1" label="有效生存期(秒)">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_pd.valid_lifetime"
            :min="1"
          />
        </n-form-item-gi>
      </template>

      <!-- DNS Servers -->
      <n-form-item-gi span="4" label="DNS 服务器">
        <n-input
          v-model:value="dnsServersStr"
          placeholder="IPv6 DNS 地址, 逗号分隔"
        />
      </n-form-item-gi>
    </template>
  </n-grid>
</template>
