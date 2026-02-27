<script lang="ts" setup>
import { computed, ref } from "vue";
import type { DHCPv6OfferInfo } from "@landscape-router/types/api/schemas";
import type {
  DHCPv6AddressItem,
  DHCPv6PrefixItem,
} from "@landscape-router/types/api/schemas";
import { useFrontEndStore } from "@/stores/front_end_config";
import { usePreferenceStore } from "@/stores/preference";
import { useEnrolledDeviceStore } from "@/stores/enrolled_device";

const prefStore = usePreferenceStore();
const frontEndStore = useFrontEndStore();
const enrolledDeviceStore = useEnrolledDeviceStore();

type Props = {
  info: DHCPv6OfferInfo;
  iface_name: string;
};

const props = defineProps<Props>();

function mac_as_string(mac: unknown): string {
  return mac as string;
}

function active_time(item: DHCPv6AddressItem | DHCPv6PrefixItem): number {
  return item.relative_active_time * 1000 + props.info.boot_time;
}

function expire_time(item: DHCPv6AddressItem | DHCPv6PrefixItem): number {
  const lifetime = "valid_lifetime" in item ? item.valid_lifetime : 0;
  return (item.relative_active_time + lifetime) * 1000 + props.info.boot_time;
}

function remaining_ms(item: DHCPv6AddressItem | DHCPv6PrefixItem): number {
  return expire_time(item) - new Date().getTime();
}

const show_addresses = computed(() => {
  return props.info.offered_addresses.map((item) => ({
    ...item,
    mac_str: item.mac ? mac_as_string(item.mac) : undefined,
    real_active_time: active_time(item),
    real_remaining: remaining_ms(item),
  }));
});

const show_prefixes = computed(() => {
  return props.info.delegated_prefixes.map((item) => ({
    ...item,
    real_active_time: active_time(item),
    real_remaining: remaining_ms(item),
  }));
});
</script>

<template>
  <n-card size="small" :title="iface_name">
    <!-- IA_NA Addresses -->
    <template v-if="show_addresses.length > 0">
      <n-divider title-placement="left" style="margin: 4px 0">
        IA_NA 地址分配
      </n-divider>
      <n-table :bordered="true" size="small" striped>
        <thead>
          <tr>
            <th style="text-align: center">主机名</th>
            <th style="text-align: center">MAC</th>
            <th style="text-align: center">IPv6 地址</th>
            <th style="text-align: center">请求时间</th>
            <th style="text-align: center">剩余租期 (s)</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(item, idx) in show_addresses" :key="idx">
            <td style="text-align: center">
              {{
                item.mac_str
                  ? enrolledDeviceStore.GET_NAME_WITH_FALLBACK(
                      item.mac_str,
                      item.hostname,
                    )
                  : (item.hostname ?? "-")
              }}
            </td>
            <td style="text-align: center">
              {{ item.mac_str ? frontEndStore.MASK_INFO(item.mac_str) : "-" }}
            </td>
            <td style="text-align: center">
              {{ frontEndStore.MASK_INFO(item.ip) }}
            </td>
            <td style="text-align: center">
              <n-time
                :time="item.real_active_time"
                :time-zone="prefStore.timezone"
              />
            </td>
            <td style="text-align: center">
              <n-flex justify="center" v-if="item.is_static"> 静态分配 </n-flex>
              <n-countdown
                v-else
                :duration="item.real_remaining"
                :active="true"
              />
            </td>
          </tr>
        </tbody>
      </n-table>
    </template>

    <!-- IA_PD Prefixes -->
    <template v-if="show_prefixes.length > 0">
      <n-divider title-placement="left" style="margin: 4px 0">
        IA_PD 前缀委派
      </n-divider>
      <n-table :bordered="true" size="small" striped>
        <thead>
          <tr>
            <th style="text-align: center">DUID</th>
            <th style="text-align: center">委派前缀</th>
            <th style="text-align: center">前缀长度</th>
            <th style="text-align: center">请求时间</th>
            <th style="text-align: center">剩余租期 (s)</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(item, idx) in show_prefixes" :key="idx">
            <td style="text-align: center">
              {{ item.duid ? item.duid.substring(0, 16) + "..." : "-" }}
            </td>
            <td style="text-align: center">
              {{ frontEndStore.MASK_INFO(item.prefix) }}
            </td>
            <td style="text-align: center">/{{ item.prefix_len }}</td>
            <td style="text-align: center">
              <n-time
                :time="item.real_active_time"
                :time-zone="prefStore.timezone"
              />
            </td>
            <td style="text-align: center">
              <n-countdown :duration="item.real_remaining" :active="true" />
            </td>
          </tr>
        </tbody>
      </n-table>
    </template>

    <n-empty
      v-if="show_addresses.length === 0 && show_prefixes.length === 0"
      description="暂无 DHCPv6 分配记录"
      style="padding: 20px 0"
    />
  </n-card>
</template>
