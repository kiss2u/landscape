<script setup lang="ts">
import { ref } from "vue";
import { FlowConfig } from "@/rust_bindings/common/flow";
import FlowEditModal from "@/components/flow/FlowEditModal.vue";
import DnsRuleDrawer from "@/components/dns/DnsRuleDrawer.vue";
import { useFrontEndStore } from "@/stores/front_end_config";
import { mask_string } from "@/lib/common";
import { del_flow_rules } from "@/api/flow";

const frontEndStore = useFrontEndStore();

interface Props {
  config: FlowConfig;
}

const props = defineProps<Props>();

const emit = defineEmits(["refresh"]);

const show_edit = ref(false);
const show_dns_rule = ref(false);
const show_ip_rule = ref(false);

async function refresh() {
  emit("refresh");
}

async function del() {
  if (props.config.id) {
    await del_flow_rules(props.config.id);
    await refresh();
  }
}
</script>

<template>
  <n-card size="small" :title="`ID: ${config.flow_id}`" :hoverable="true">
    <template #header-extra>
      <n-flex>
        <n-button secondary @click="show_edit = true" size="small">
          修改配置
        </n-button>
        <n-button secondary @click="show_dns_rule = true" size="small">
          DNS
        </n-button>
        <n-button secondary @click="show_ip_rule = true" size="small">
          目标 IP
        </n-button>
        <n-popconfirm @positive-click="del">
          <template #trigger>
            <n-button type="error" secondary size="small">删除</n-button>
          </template>
          确定删除吗?
        </n-popconfirm>
      </n-flex>
    </template>

    <template #footer>
      <!-- <n-flex>
        <n-tag :bordered="false" v-for="rule in config.flow_match_rules">
          {{ `${rule.ip} - ${rule.qos ?? "N/A"} - ${rule.vlan_id ?? "N/A"}` }}
        </n-tag>
      </n-flex>
    </template>
    <template #action>
      <n-flex>
        <n-tag
          :bordered="false"
          v-for="target in config.packet_handle_iface_name"
          :type="`${target.t === FlowTargetTypes.NETNS ? 'info' : ''}`"
        >
          {{
            target.t === FlowTargetTypes.NETNS
              ? target.container_name
              : target.name
          }}
        </n-tag>
      </n-flex> -->
    </template>

    <n-descriptions :column="1" label-placement="left">
      <n-descriptions-item label="状态">
        {{ config.enable ? "启用" : "禁用" }}
      </n-descriptions-item>
      <n-descriptions-item label="备注">
        {{
          frontEndStore.presentation_mode
            ? mask_string(config.remark)
            : config.remark
        }}
      </n-descriptions-item>
    </n-descriptions>
    <!-- {{ config }} -->
    <FlowEditModal
      @refresh="refresh"
      v-model:show="show_edit"
      :rule_id="props.config.id"
    >
    </FlowEditModal>
    <DnsRuleDrawer v-model:show="show_dns_rule" :flow_id="props.config.flow_id">
    </DnsRuleDrawer>
    <WanIpRuleDrawer
      :flow_id="props.config.flow_id"
      v-model:show="show_ip_rule"
    />
  </n-card>
</template>
