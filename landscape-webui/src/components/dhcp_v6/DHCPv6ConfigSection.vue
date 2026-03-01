<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { DHCPv6ServerConfig } from "@landscape-router/types/api/schemas";

const { t } = useI18n();

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
        delegate_prefix_len: 64,
        preferred_lifetime: 3600,
        valid_lifetime: 7200,
      };
    } else {
      config.value.ia_pd = undefined;
    }
  },
});

const showMFlagWarning = computed(() => {
  return enabled.value && !props.managed_address_config;
});
</script>

<template>
  <n-grid :x-gap="12" :y-gap="8" cols="4" item-responsive>
    <n-form-item-gi span="2" :label="t('dhcp_v6.enable_dhcpv6')">
      <n-switch v-model:value="enabled">
        <template #checked> {{ t("common.enable") }} </template>
        <template #unchecked> {{ t("common.disable") }} </template>
      </n-switch>
    </n-form-item-gi>

    <n-form-item-gi span="4" v-if="showMFlagWarning">
      <n-alert type="warning" :bordered="false">
        {{ t("dhcp_v6.m_flag_warning") }}
      </n-alert>
    </n-form-item-gi>

    <template v-if="enabled && config">
      <!-- IA_NA Section -->
      <n-form-item-gi span="4" :label="t('dhcp_v6.ia_na')">
        <n-switch v-model:value="ia_na_enabled">
          <template #checked> {{ t("common.enable") }} </template>
          <template #unchecked> {{ t("common.disable") }} </template>
        </n-switch>
      </n-form-item-gi>

      <template v-if="config.ia_na">
        <n-form-item-gi span="2">
          <template #label>
            <Notice>
              {{ t("dhcp_v6.max_prefix_len") }}
              <template #msg>
                {{ t("dhcp_v6.max_prefix_len_desc") }}
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

        <n-form-item-gi span="2" :label="t('dhcp_v6.pool_start')">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_na.pool_start"
            :min="1"
          />
        </n-form-item-gi>

        <n-form-item-gi span="2" :label="t('dhcp_v6.pool_end')">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_na.pool_end"
            :min="config.ia_na.pool_start + 1"
            :placeholder="t('dhcp_v6.pool_end_placeholder')"
            clearable
          />
        </n-form-item-gi>

        <n-form-item-gi span="1" :label="t('dhcp_v6.preferred_lifetime')">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_na.preferred_lifetime"
            :min="1"
          />
        </n-form-item-gi>

        <n-form-item-gi span="1" :label="t('dhcp_v6.valid_lifetime')">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_na.valid_lifetime"
            :min="1"
          />
        </n-form-item-gi>
      </template>

      <!-- IA_PD Section -->
      <n-form-item-gi span="4" :label="t('dhcp_v6.ia_pd')">
        <n-switch v-model:value="ia_pd_enabled">
          <template #checked> {{ t("common.enable") }} </template>
          <template #unchecked> {{ t("common.disable") }} </template>
        </n-switch>
      </n-form-item-gi>

      <template v-if="config.ia_pd">
        <n-form-item-gi span="2" :label="t('dhcp_v6.delegate_prefix_length')">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_pd.delegate_prefix_len"
            :min="1"
            :max="128"
          />
        </n-form-item-gi>

        <n-form-item-gi span="1" :label="t('dhcp_v6.preferred_lifetime')">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_pd.preferred_lifetime"
            :min="1"
          />
        </n-form-item-gi>

        <n-form-item-gi span="1" :label="t('dhcp_v6.valid_lifetime')">
          <n-input-number
            style="flex: 1"
            v-model:value="config.ia_pd.valid_lifetime"
            :min="1"
          />
        </n-form-item-gi>
      </template>
    </template>
  </n-grid>
</template>
