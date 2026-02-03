<script setup lang="ts">
import { onMounted, ref } from "vue";
import { usePreferenceStore } from "@/stores/preference";
import { get_init_config } from "@/api/sys/config";
import { useMessage } from "naive-ui";
import { useI18n } from "vue-i18n";

const { t } = useI18n();
const prefStore = usePreferenceStore();
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
    await prefStore.loadPreferenceForEdit();
  } catch (e) {
    message.error("加载配置失败");
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

        <n-form label-placement="left" label-width="100">
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
              style="max-width: 300px"
            />
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
