<script setup lang="ts">
import { get_docker_container_summarys } from "@/api/docker";
import { get_wan_ifaces } from "@/api/iface";
import { get_all_iface_pppd_config } from "@/api/service_pppd";
import type {
  FlowTarget,
  WeightedFlowTarget,
} from "@landscape-router/types/api/schemas";
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

const target_rules = defineModel<WeightedFlowTarget[]>("target_rules", {
  required: true,
});

const iface_wans = ref<any[]>([]);
const docker_containers = ref<any[]>([]);
const pppd_services = ref<any[]>([]);

onMounted(async () => {
  await refresh_wan_ifaces();
});

async function refresh_wan_ifaces() {
  iface_wans.value = await get_wan_ifaces();
  docker_containers.value = await get_docker_container_summarys();
  pppd_services.value = await get_all_iface_pppd_config();
}

const iface_wan_options = computed(() => {
  let iface = iface_wans.value.map((e) => ({
    label: e.name,
    value: e.name,
  }));
  let pppd_iface = pppd_services.value.map((e) => ({
    label: e.iface_name,
    value: e.iface_name,
  }));
  return [...iface, ...pppd_iface];
});

const docker_options = computed(() =>
  docker_containers.value.map((e) => {
    let name = e.Names[0] ?? "";
    if (name.startsWith("/")) {
      name = name.slice(1);
    }
    return {
      label: name,
      value: name,
    };
  }),
);

enum FlowTargetEnum {
  Interface = "interface",
  NetNS = "netns",
}

function onCreate(): WeightedFlowTarget {
  return {
    target: { t: "interface", name: "" },
    weight: 1,
  };
}

function target_type_option(): any[] {
  return [
    {
      label: t("flow.target_rule.type_wan"),
      value: "interface",
    },
    {
      label: t("flow.target_rule.type_docker"),
      value: "netns",
    },
  ];
}

function handleUpdateValue(value: FlowTarget["t"], index: number) {
  const weight = target_rules.value[index]?.weight ?? 1;
  if (value == FlowTargetEnum.Interface) {
    target_rules.value[index] = {
      target: {
        t: FlowTargetEnum.Interface,
        name: "",
      },
      weight,
    };
  } else {
    target_rules.value[index] = {
      target: {
        t: FlowTargetEnum.NetNS,
        container_name: "",
      },
      weight,
    };
  }
}
</script>

<template>
  <!-- {{ docker_options }} -->
  <!-- {{ docker_containers }} -->
  <n-dynamic-input :min="0" v-model:value="target_rules" :on-create="onCreate">
    <template #create-button-default>
      {{ t("flow.target_rule.add_target_rule") }}
    </template>
    <template #default="{ value, index }">
      <n-input-group>
        <n-select
          :style="{ width: '24%' }"
          v-model:value="value.target.t"
          @update:value="handleUpdateValue($event, index)"
          :options="target_type_option()"
        />

        <n-select
          v-if="value.target.t == 'interface'"
          v-model:value="value.target.name"
          :style="{ width: '56%' }"
          :options="iface_wan_options"
          :placeholder="t('flow.target_rule.iface_placeholder')"
        />
        <n-select
          v-else-if="value.target.t == 'netns'"
          v-model:value="value.target.container_name"
          :style="{ width: '56%' }"
          :options="docker_options"
          :placeholder="t('flow.target_rule.container_placeholder')"
        />

        <n-input-number
          v-model:value="value.weight"
          :style="{ width: '20%' }"
          :min="0"
          :step="1"
          :show-button="false"
          :placeholder="t('flow.target_rule.weight_placeholder')"
        />
      </n-input-group>
    </template>
  </n-dynamic-input>
</template>
