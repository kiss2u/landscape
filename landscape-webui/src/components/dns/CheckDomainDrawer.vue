<script setup lang="ts">
import { ref } from "vue";
import { useMessage } from "naive-ui";
import { SearchLocate } from "@vicons/carbon";
import { CheckDnsReq, CheckDnsResult } from "@/rust_bindings/dns";
import { check_domain } from "@/api/dns_service";
const message = useMessage();

interface Props {
  flow_id?: number;
}

const props = withDefaults(defineProps<Props>(), {
  flow_id: 0,
});

const show = defineModel<boolean>("show", { required: true });
const req = ref<CheckDnsReq>({
  flow_id: 0,
  domain: "",
  record_type: "A",
});
const result = ref<CheckDnsResult>({
  config: undefined,
  records: null,
  cache_records: null,
});

async function init_req() {
  req.value = {
    flow_id: props.flow_id,
    domain: "",
    record_type: "A",
  };
  result.value = {
    config: undefined,
    records: null,
    cache_records: null,
  };
}
const options = [
  {
    label: "A",
    value: "A",
  },
  {
    label: "AAAA",
    value: "AAAA",
  },
];

const loading = ref(false);
async function query() {
  if (req.value.domain !== "") {
    loading.value = true;
    try {
      result.value = await check_domain(req.value);
    } finally {
      loading.value = false;
    }
  } else {
    message.info("请输入待查询域名");
  }
}
</script>

<template>
  <n-drawer
    @after-enter="init_req()"
    @after-leave="init_req()"
    v-model:show="show"
    width="500px"
    placement="right"
    :mask-closable="false"
  >
    <n-drawer-content title="测试域名查询 (结果不会被缓存)" closable>
      <n-flex style="height: 100%" vertical>
        <n-spin :show="loading">
          <n-input-group>
            <n-select
              :style="{ width: '33%' }"
              v-model:value="req.record_type"
              :options="options"
            />
            <n-input v-model:value="req.domain" />

            <n-button @click="query">
              <template #icon>
                <n-icon>
                  <SearchLocate />
                </n-icon>
              </template>
            </n-button>
          </n-input-group>
        </n-spin>

        <n-scrollbar>
          <n-flex vertical>
            <n-descriptions
              v-if="result.config"
              bordered
              title="匹配配置"
              label-placement="top"
              :column="3"
            >
              <n-descriptions-item label="名称">
                {{ result.config.name }}
              </n-descriptions-item>
              <n-descriptions-item label="启用">
                {{ result.config.enable }}
              </n-descriptions-item>
              <n-descriptions-item label="流量标记">
                {{ result.config.mark }}
              </n-descriptions-item>
              <n-descriptions-item label="DNS 处理方式" :span="2">
                {{ result.config.resolve_mode }}
              </n-descriptions-item>
              <n-descriptions-item label="匹配规则">
                {{ result.config.source }}
              </n-descriptions-item>
            </n-descriptions>
            <n-divider title-placement="left"> DNS 上游查询结果 </n-divider>
            <n-flex v-if="result.records">
              <n-flex v-for="each in result.records">
                {{ each }}
              </n-flex>
            </n-flex>
            <n-divider title-placement="left"> DNS 内部缓存结果 </n-divider>
            <n-flex v-if="result.cache_records">
              <n-flex v-for="each in result.cache_records">
                {{ each }}
              </n-flex>
            </n-flex>
          </n-flex>
        </n-scrollbar>
      </n-flex>
    </n-drawer-content>
  </n-drawer>
</template>
