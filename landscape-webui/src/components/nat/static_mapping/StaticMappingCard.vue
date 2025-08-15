<script setup lang="ts">
import { delete_static_nat_mapping } from "@/api/static_nat_mapping";
import { StaticNatMappingConfig } from "@/rust_bindings/common/nat";
import { useThemeVars } from "naive-ui";
import { computed, ref } from "vue";
import { DotMark } from "@vicons/carbon";

const rule = defineModel<StaticNatMappingConfig>("rule", { required: true });

const show_edit_modal = ref(false);
const themeVars = ref(useThemeVars());

const emit = defineEmits(["refresh"]);

async function del() {
  if (rule.value.id !== null) {
    await delete_static_nat_mapping(rule.value.id);
    emit("refresh");
  }
}

const title = computed(() => {
  const wan_iface_name = rule.value.wan_iface_name
    ? `${rule.value.wan_iface_name}:`
    : "any wan";

  const target_str =
    rule.value.lan_ip === "0.0.0.0" || rule.value.lan_ip === "::"
      ? `Route:${rule.value.lan_port}`
      : `${rule.value.lan_ip}:${rule.value.lan_port}`;
  return `${wan_iface_name}:${rule.value.wan_port} => ${target_str}`;
});
</script>
<template>
  <n-flex>
    <n-card size="small" style="flex: 1; min-height: 150px">
      <template #header>
        <n-flex align="center" :wrap="false">
          <n-icon :color="rule.enable ? themeVars.successColor : ''" size="14">
            <DotMark />
          </n-icon>
          {{ title }}
        </n-flex>
      </template>
      <!-- {{ rule }} -->

      <n-descriptions bordered label-placement="top" :column="2">
        <!-- <n-descriptions-item label="启用">
          <n-tag :bordered="false" :type="rule.enable ? 'success' : ''">
            {{ rule.enable }}
          </n-tag>
        </n-descriptions-item> -->
        <!-- <n-descriptions-item label="流量标记">
          {{ rule.mark }}
        </n-descriptions-item> -->
        <n-descriptions-item label="备注">
          {{ rule.remark === "" ? "无备注" : rule.remark }}
        </n-descriptions-item>
        <!-- <n-descriptions-item label="匹配规则">
          {{ rule.items }}
        </n-descriptions-item> -->
      </n-descriptions>
      <template #header-extra>
        <n-flex>
          <n-button type="warning" secondary @click="show_edit_modal = true">
            编辑
          </n-button>

          <n-popconfirm @positive-click="del()">
            <template #trigger>
              <n-button type="error" secondary @click=""> 删除 </n-button>
            </template>
            确定删除吗
          </n-popconfirm>
        </n-flex>
      </template>
    </n-card>
    <MappingEditModal
      @refresh="emit('refresh')"
      :rule_id="rule.id"
      v-model:show="show_edit_modal"
    >
    </MappingEditModal>
  </n-flex>
</template>
