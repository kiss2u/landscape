<script setup lang="ts">
import { ref, reactive, onMounted, h, computed, watch } from "vue";
import { DnsMetric, get_dns_history } from "@/api/metric";
import { NDataTable, NTag, NTime, NButton, NSpace, NInput, NDatePicker, NIcon, NTooltip, NTabs, NTabPane, NSelect, NInputNumber } from "naive-ui";
import type { DataTableColumns } from 'naive-ui'
import { Refresh, TrashOutline, HelpCircleOutline, TimeOutline } from "@vicons/ionicons5";
import { useDebounceFn } from "@vueuse/core";
import DNSDashboard from "./DNSDashboard.vue";

const activeTab = ref('dashboard');
const dashboardRef = ref<any>(null);

const data = ref<DnsMetric[]>([]);
const loading = ref(false);

const DEFAULT_TIME_WINDOW = 20 * 60 * 1000; // 20 minutes

const searchParams = reactive({
    domain: '',
    src_ip: '',
    query_type: null as string | null,
    status: null as string | null,
    min_duration_ms: null as number | null,
    max_duration_ms: null as number | null,
    timeRange: [Date.now() - DEFAULT_TIME_WINDOW, Date.now()] as [number, number] | null,
    sort_key: 'time',
    sort_order: 'desc' as 'asc' | 'desc'
});

const pagination = reactive({
    page: 1,
    pageSize: 15, // Default 15
    itemCount: 0,
    showSizePicker: true,
    pageSizes: [15, 30, 50, 100],
    onChange: (page: number) => {
        pagination.page = page;
        loadData();
    },
    onUpdatePageSize: (pageSize: number) => {
        pagination.pageSize = pageSize;
        pagination.page = 1;
        loadData();
    }
})

const formatIp = (ip: string) => {
    if (ip.startsWith('::ffff:')) {
        return ip.substring(7);
    }
    return ip;
};

const queryTypeOptions = [
    { label: 'All Types', value: null },
    { label: 'A (IPv4)', value: 'A' },
    { label: 'AAAA (IPv6)', value: 'AAAA' },
    { label: 'CNAME', value: 'CNAME' },
    { label: 'MX', value: 'MX' },
    { label: 'TXT', value: 'TXT' },
    { label: 'NS', value: 'NS' },
    { label: 'PTR', value: 'PTR' },
    { label: 'SOA', value: 'SOA' },
    { label: 'SRV', value: 'SRV' },
];

const statusOptions = [
    { label: 'All Status', value: null },
    { label: 'Hit (Cache)', value: 'hit' },
    { label: 'Normal', value: 'normal' },
    { label: 'Block', value: 'block' },
    { label: 'Local', value: 'local' },
    { label: 'NXDomain', value: 'nxdomain' },
    { label: 'Error', value: 'error' },
];

const columns = computed<DataTableColumns<DnsMetric>>(() => [
  {
    title: 'Time',
    key: 'report_time',
    width: 200,
    sorter: true,
    sortOrder: searchParams.sort_key === 'time' ? (searchParams.sort_order === 'asc' ? 'ascend' : 'descend') : false,
    render(row) {
        return h(NTime, { time: Number(row.report_time), type: "datetime" })
    }
  },
  { 
    title: 'Domain', 
    key: 'domain',
    sorter: true,
    sortOrder: searchParams.sort_key === 'domain' ? (searchParams.sort_order === 'asc' ? 'ascend' : 'descend') : false,
    ellipsis: {
      tooltip: true
    }
  },
  { 
    title: 'Type', 
    key: 'query_type',
    width: 80,
    render(row) {
      return h(NTag, { type: 'info', size: 'small' }, { default: () => row.query_type })
    }
  },
  { 
    title: 'Src IP', 
    key: 'src_ip', 
    width: 140,
    render(row) {
        return formatIp(String(row.src_ip))
    }
  },
  { 
    title: 'Resp Code', 
    key: 'response_code', 
    width: 150,
    render(row) {
      const code = String(row.response_code || '').toLowerCase();
      const isOk = code === 'noerror' || code === 'no error';
      return h(NTag, { 
        type: isOk ? 'success' : 'error', 
        size: 'small',
        style: { minWidth: '80px', justifyContent: 'center' }
      }, { default: () => row.response_code })
    }
  },
  { 
    title: 'Status', 
    key: 'status', 
    width: 110,
    render(row) {
      const statusMap: Record<string, { type: any, label: string }> = {
          'local': { type: 'success', label: 'Local' },
          'block': { type: 'warning', label: 'Block' },
          'hit': { type: 'info', label: 'Hit' },
          'nxdomain': { type: 'default', label: 'NXDomain' },
          'normal': { type: 'default', label: 'Normal' },
          'error': { type: 'error', label: 'Error' }
      };
      const s = statusMap[row.status] || { type: 'default', label: row.status };
      return h(NTag, { 
        type: s.type, 
        size: 'small', 
        bordered: false,
        style: { minWidth: '70px', justifyContent: 'center' }
      }, { default: () => s.label })
    }
  },
  { 
    title: 'Duration (ms)', 
    key: 'duration_ms', 
    width: 120,
    sorter: true,
    sortOrder: searchParams.sort_key === 'duration' ? (searchParams.sort_order === 'asc' ? 'ascend' : 'descend') : false
  },
  { 
      title: 'Answers', 
      key: 'answers',
      ellipsis: {
        tooltip: true
      },
      render(row) {
          if (!row.answers || row.answers.length === 0) return "-";
          return row.answers.map(formatIp).join(", ");
      }
  }
])

const loadData = async (resetPage = false) => {
  if (resetPage) pagination.page = 1;
    if (activeTab.value === 'dashboard') {
        dashboardRef.value?.refresh();
        return;
    }

    loading.value = true;
    try {
    const params: any = { 
        limit: pagination.pageSize,
        offset: (pagination.page - 1) * pagination.pageSize,
        sort_key: searchParams.sort_key, 
        sort_order: searchParams.sort_order 
    };

    if (searchParams.domain && searchParams.domain.trim()) {
      params.domain = searchParams.domain.trim();
    }
    if (searchParams.src_ip && searchParams.src_ip.trim()) {
      params.src_ip = searchParams.src_ip.trim();
    }
    if (searchParams.query_type) {
      params.query_type = searchParams.query_type;
    }
    if (searchParams.status) {
      params.status = searchParams.status;
    }
    if (searchParams.min_duration_ms !== null && searchParams.min_duration_ms !== undefined) {
      params.min_duration_ms = searchParams.min_duration_ms;
    }
    if (searchParams.max_duration_ms !== null && searchParams.max_duration_ms !== undefined) {
      params.max_duration_ms = searchParams.max_duration_ms;
    }
    
    if (Array.isArray(searchParams.timeRange) && searchParams.timeRange.length === 2) {
        params.start_time = searchParams.timeRange[0];
        params.end_time = searchParams.timeRange[1];
    }

    const res = await get_dns_history(params);
    data.value = res.items;
    pagination.itemCount = res.total;
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
};

const debouncedLoadData = useDebounceFn(() => {
    loadData(true);
}, 500);

watch(
    () => [searchParams.domain, searchParams.src_ip, searchParams.query_type, searchParams.status, searchParams.min_duration_ms, searchParams.max_duration_ms],
    () => debouncedLoadData()
);

watch(
    () => searchParams.timeRange,
    () => loadData(true)
);

const handleSorterChange = (sorter: any) => {
    if (!sorter || !sorter.order) {
        searchParams.sort_key = 'time';
        searchParams.sort_order = 'desc';
    } else {
        const keyMap: Record<string, string> = {
            'report_time': 'time',
            'domain': 'domain',
            'duration_ms': 'duration'
        };
        searchParams.sort_key = keyMap[sorter.columnKey] || 'time';
        searchParams.sort_order = sorter.order === 'ascend' ? 'asc' : 'desc';
    }
    loadData(true);
}

const shortcuts = {
    '20m': () => [Date.now() - 20 * 60 * 1000, Date.now()] as [number, number],
    '1h': () => [Date.now() - 60 * 60 * 1000, Date.now()] as [number, number],
    '12h': () => [Date.now() - 12 * 60 * 60 * 1000, Date.now()] as [number, number],
    '24h': () => [Date.now() - 24 * 60 * 60 * 1000, Date.now()] as [number, number]
};

const syncToNow = () => {
    if (searchParams.timeRange && searchParams.timeRange.length === 2) {
        const duration = searchParams.timeRange[1] - searchParams.timeRange[0];
        const now = Date.now();
        searchParams.timeRange = [now - duration, now];
    } else {
        searchParams.timeRange = [Date.now() - DEFAULT_TIME_WINDOW, Date.now()];
    }
    loadData(true);
}

watch(
    () => activeTab.value,
    (tab) => {
        if (tab === 'dashboard') {
            dashboardRef.value?.refresh();
        } else {
            loadData(true);
        }
    }
)

const handleReset = () => {
    searchParams.domain = '';
    searchParams.src_ip = '';
    searchParams.query_type = null;
    searchParams.status = null;
    searchParams.min_duration_ms = null;
    searchParams.max_duration_ms = null;
    searchParams.timeRange = [Date.now() - DEFAULT_TIME_WINDOW, Date.now()];
    searchParams.sort_key = 'time';
    searchParams.sort_order = 'desc';
    loadData(true);
}

onMounted(() => {
  loadData();
});
</script>

<template>
  <div style="width: 100%; padding: 12px">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px">
      <h3 style="margin: 0; font-weight: 500; font-size: 1.1rem">DNS History</h3>
      <n-space :size="8">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-icon size="18" style="vertical-align: middle; cursor: help; color: rgba(0,0,0,0.35)">
              <HelpCircleOutline />
            </n-icon>
          </template>
          Auto-search (500ms debounce)
        </n-tooltip>
        <n-button circle size="tiny" @click="loadData(true)" tertiary>
          <template #icon>
            <n-icon><Refresh /></n-icon>
          </template>
        </n-button>
      </n-space>
    </div>

    <n-space style="margin-bottom: 10px" align="center" :size="[8, 8]" :wrap="false">
       <n-input v-model:value="searchParams.domain" size="small" placeholder="Domain" clearable style="width: 200px" v-if="activeTab === 'history'"/>
       <n-input v-model:value="searchParams.src_ip" size="small" placeholder="IP" clearable style="width: 140px" v-if="activeTab === 'history'"/>
       <n-select 
         v-model:value="searchParams.query_type" 
         size="small" 
         :options="queryTypeOptions" 
         placeholder="Type" 
         clearable 
         style="width: 130px" 
         v-if="activeTab === 'history'"
       />
       <n-select 
         v-model:value="searchParams.status" 
         size="small" 
         :options="statusOptions" 
         placeholder="Status" 
         clearable 
         style="width: 130px" 
         v-if="activeTab === 'history'"
       />
       <n-input-number 
         v-model:value="searchParams.min_duration_ms" 
         size="small" 
         placeholder="Min ms" 
         clearable 
         :min="0"
         :show-button="false"
         style="width: 100px" 
         v-if="activeTab === 'history'"
       />
       <n-input-number 
         v-model:value="searchParams.max_duration_ms" 
         size="small" 
         placeholder="Max ms" 
         clearable 
         :min="0"
         :show-button="false"
         style="width: 100px" 
         v-if="activeTab === 'history'"
       />
       <n-date-picker v-model:value="searchParams.timeRange" size="small" type="datetimerange" clearable :shortcuts="shortcuts" placeholder="Time Range" style="width: 320px"/>
       <n-tooltip trigger="hover">
          <template #trigger>
            <n-button strong secondary size="small" @click="syncToNow" type="info">
               <template #icon>
                 <n-icon><TimeOutline /></n-icon>
               </template>
               Now
            </n-button>
          </template>
          Sync time range to now (preserve duration)
       </n-tooltip>
       <n-button @click="handleReset" size="small" secondary>
          <template #icon><n-icon><TrashOutline /></n-icon></template>
          Reset
       </n-button>
    </n-space>

    <n-tabs v-model:value="activeTab" type="line" animated>
      <n-tab-pane name="dashboard" tab="Dashboard">
        <DNSDashboard ref="dashboardRef" :time-range="searchParams.timeRange" />
      </n-tab-pane>
      <n-tab-pane name="history" tab="Query Log">
        <n-data-table 
          remote
          :columns="columns" 
          :data="data" 
          :loading="loading"
          :pagination="pagination"
          @update:sorter="handleSorterChange"
          size="small"
          :row-key="(row) => row.report_time + row.domain + row.flow_id"
          :bordered="false"
          class="dns-history-table"
        />
      </n-tab-pane>
    </n-tabs>
  </div>
</template>

<style scoped>
.dns-history-table :deep(.n-data-table-wrapper) {
  border-radius: 8px;
}
</style>
