<script setup lang="ts">
import { onMounted, ref } from "vue";
import { usePreferenceStore } from "@/stores/preference";
import { useMetricConfigStore } from "@/stores/metric_config";
import { get_init_config } from "@/api/sys/config";
import { useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const prefStore = usePreferenceStore();
const metricStore = useMetricConfigStore();
const message = useMessage();
const loading = ref(false);

const languageOptions = [
  { label: "简体中文", value: "zh-CN" },
  { label: "English", value: "en" },
];

const themeOptions = [
  { label: "深色模式", value: "dark" },
  { label: "浅色模式", value: "light" },
];

const timezoneOptions = (Intl as any)
  .supportedValuesOf("timeZone")
  .map((tz: string) => ({
    label: tz,
    value: tz,
  }));

onMounted(async () => {
  loading.value = true;
  try {
    await Promise.all([
      prefStore.loadPreferenceForEdit(),
      metricStore.loadMetricConfig(),
    ]);
  } catch (e) {
    message.error("加载配置失败");
    console.error(e);
  } finally {
    loading.value = false;
  }
});

async function handleSave() {
  try {
    await prefStore.savePreference();
    message.success("保存成功");
  } catch (e: any) {
    if (e.response?.status === 409) {
      message.error("配置冲突，请刷新后重试");
    } else {
      message.error("保存失败: " + e.message);
    }
  }
}

async function handleSaveMetric() {
  try {
    await metricStore.saveMetricConfig();
    message.success("指标配置保存成功");
  } catch (e: any) {
    if (e.response?.status === 409) {
      message.error("指标配置冲突，请刷新后重试");
    } else {
      message.error("保存失败: " + e.message);
    }
  }
}

async function export_file() {
  await get_init_config();
}
</script>

<template>
  <div class="config-container">
    <n-space vertical size="large">
      <n-card title="系统偏好设置" segmented>
        <template #header-extra>
          <n-button type="primary" :loading="loading" @click="handleSave">
            保存设置
          </n-button>
        </template>

        <n-form label-placement="left" label-width="120">
          <n-form-item label="语言设定">
            <n-select
              v-model:value="prefStore.language"
              :options="languageOptions"
              style="max-width: 300px"
            />
          </n-form-item>
          <n-form-item label="外观主题">
            <n-select
              v-model:value="prefStore.theme"
              :options="themeOptions"
              disabled
              placeholder="浅色模式适配中，暂不可用"
              style="max-width: 300px"
            />
          </n-form-item>
          <n-form-item label="系统时区">
            <n-select
              v-model:value="prefStore.timezone"
              filterable
              :options="timezoneOptions"
              placeholder="请选择或搜索，例如: Asia/Shanghai"
              style="max-width: 400px"
            />
          </n-form-item>
        </n-form>
      </n-card>

      <n-card title="指标监控配置" segmented>
        <template #header-extra>
          <n-button type="primary" :loading="loading" @click="handleSaveMetric">
            保存指标配置
          </n-button>
        </template>
        <n-form label-placement="left" label-width="120">
          <n-form-item label="数据保存天数">
            <n-input-number
              v-model:value="metricStore.retentionDays"
              :min="1"
              :max="365"
              placeholder="7"
              style="width: 200px"
            />
            <template #feedback>
              历史连接和流量指标数据的保存期限（天）
            </template>
          </n-form-item>
          <n-form-item label="刷新间隔 (秒)">
            <n-input-number
              v-model:value="metricStore.flushIntervalSecs"
              :min="1"
              :max="3600"
              placeholder="5"
              style="width: 200px"
            />
            <template #feedback> 指标刷新到存储的间隔时间 </template>
          </n-form-item>
          <n-form-item label="批处理大小">
            <n-input-number
              v-model:value="metricStore.batchSize"
              :min="100"
              :max="10000"
              placeholder="2000"
              style="width: 200px"
            />
            <template #feedback> 每次写入存储的最大记录数 </template>
          </n-form-item>
          <n-form-item label="最大内存限制 (MB)">
            <n-input-number
              v-model:value="metricStore.maxMemory"
              :min="32"
              :max="4096"
              placeholder="128"
              style="width: 200px"
            />
            <template #feedback> 指标缓存允许占用的最大内存 </template>
          </n-form-item>
          <n-form-item label="并发处理线程">
            <n-input-number
              v-model:value="metricStore.maxThreads"
              :min="1"
              :max="16"
              placeholder="1"
              style="width: 200px"
            />
            <template #feedback> 用于处理指标数据的后台线程数 </template>
          </n-form-item>
        </n-form>
      </n-card>

      <n-card title="备份与导出" segmented>
        <n-p
          >你可以将当前路由器的所有配置（包括
          DNS、防火墙、网络接口等）导出为一个初始化文件，用于快速恢复或迁移。</n-p
        >
        <n-button @click="export_file" type="info" ghost>
          导出当前所有配置为 Init 文件
        </n-button>
      </n-card>
    </n-space>
  </div>
</template>

<style scoped>
.config-container {
  padding: 24px;
}
</style>
