<script setup lang="ts">
import { usePreferenceStore } from "@/stores/preference";
import { useMessage } from "naive-ui";

const prefStore = usePreferenceStore();
const message = useMessage();

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

async function handleSave() {
  try {
    await prefStore.savePreference();
    message.success("系统设置保存成功");
  } catch (e: any) {
    if (e.response?.status === 409) {
      message.error("配置冲突，请刷新后重试");
    } else {
      message.error("保存失败: " + e.message);
    }
  }
}
</script>

<template>
  <n-card title="系统偏好设置" segmented id="ui-config">
    <template #header-extra>
      <n-button type="primary" @click="handleSave"> 保存设置 </n-button>
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
</template>
