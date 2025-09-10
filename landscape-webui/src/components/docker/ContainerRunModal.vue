<script setup lang="ts">
import { run_cmd } from "@/api/docker";
import { KeyValuePair } from "@/lib/common";
import { LAND_REDIRECT_ID_KEY } from "@/lib/docker";
import { DockerCmd } from "@/rust_bindings/common/docker";
import { useDockerStore } from "@/stores/status_docker";
import { useNotification } from "naive-ui";
import { computed, ref } from "vue";

const show_model = defineModel<boolean>("show", { required: true });

const props = defineProps<{
  image_name: string;
}>();

const emit = defineEmits(["refresh"]);

const dockerStore = useDockerStore();
const notification = useNotification();

// 定义表单的状态
const formModel = ref<DockerCmd>();

async function on_modal_enter() {
  formModel.value = {
    image_name: props.image_name,
    restart: DockerRestartPolicy.NO,
    restart_max_retries: 3,
    container_name: null,
    ports: null,
    environment: null,
    volumes: null,
    labels: null,
  };
}

const save_loading = ref(false);
async function save_config() {
  if (formModel.value) {
    try {
      save_loading.value = true;
      await run_cmd(formModel.value);
      dockerStore.UPDATE_INFO();
      show_model.value = false;
    } finally {
      save_loading.value = false;
    }
  }
}

function add_redirect_id() {
  if (formModel.value) {
    if (formModel.value.labels) {
      for (const label of formModel.value.labels) {
        if (label.key === LAND_REDIRECT_ID_KEY) {
          // 提示
          notification.info({
            content: "已经存在标签了",
            duration: 2500,
            keepAliveOnHover: true,
          });
          break;
        }
      }

      formModel.value.labels.unshift({
        key: LAND_REDIRECT_ID_KEY,
        value: "",
      });
    }
  }
}

const show_add_redirect_id_btn = computed(() => {
  if (formModel.value?.labels) {
    for (const label of formModel.value.labels) {
      if (label.key === LAND_REDIRECT_ID_KEY) {
        return true;
      }
    }
  }

  return false;
});

enum DockerRestartPolicy {
  NO = "no",
  ON_FAILURE = "on-failure",
  ON_FAILURE_WITH_MAX_RETRIES = "on-failure:<max-retries>",
  ALWAYS = "always",
  UNLESS_STOPPED = "unless-stopped",
}

const restrt_options = [
  {
    label: "不自动重启",
    value: DockerRestartPolicy.NO,
  },
  {
    label: "失败时自动重启",
    value: DockerRestartPolicy.ON_FAILURE,
  },
  {
    label: "失败时自动重启（带最大重试次数）",
    value: DockerRestartPolicy.ON_FAILURE_WITH_MAX_RETRIES,
  },
  {
    label: "总是自动重启",
    value: DockerRestartPolicy.ALWAYS,
  },
  {
    label: "除非手动停止，否则自动重启",
    value: DockerRestartPolicy.UNLESS_STOPPED,
  },
];
</script>

<template>
  <n-modal
    :auto-focus="false"
    v-model:show="show_model"
    @after-enter="on_modal_enter"
  >
    <n-card
      style="width: 600px"
      :title="`运行镜像: ${props.image_name}`"
      :bordered="false"
      size="small"
      role="dialog"
      aria-modal="true"
    >
      <n-form v-if="formModel" :model="formModel" label-width="120px">
        <n-form-item label="镜像名称" path="imageName">
          <n-input
            :disabled="true"
            v-model:value="formModel.image_name"
            placeholder="请输入镜像名称"
          />
        </n-form-item>
        <n-form-item label="重启策略" path="containerName">
          <n-input-group>
            <n-select
              v-model:value="formModel.restart"
              :options="restrt_options"
            />
            <n-input-number
              v-if="
                formModel.restart ===
                DockerRestartPolicy.ON_FAILURE_WITH_MAX_RETRIES
              "
              v-model:value="formModel.restart_max_retries"
              placeholder=""
            />
          </n-input-group>
        </n-form-item>
        <n-form-item label="容器名称" path="containerName">
          <n-input
            v-model:value="formModel.container_name"
            placeholder="请输入容器名称 (可选)"
          />
        </n-form-item>
        <n-form-item label="端口映射" path="ports">
          <n-dynamic-input
            v-model:value="formModel.ports"
            preset="pair"
            separator=":"
            key-placeholder="主机端口"
            value-placeholder="容器端口"
          />
        </n-form-item>
        <n-form-item label="环境变量" path="environment">
          <n-dynamic-input
            v-model:value="formModel.environment"
            preset="pair"
            separator=":"
            key-placeholder="变量名"
            value-placeholder="变量值"
          />
        </n-form-item>
        <n-form-item label="卷映射" path="volumes">
          <n-dynamic-input
            v-model:value="formModel.volumes"
            preset="pair"
            separator=":"
            key-placeholder="主机目录"
            value-placeholder="容器目录"
          />
        </n-form-item>
        <n-form-item label="标签">
          <n-flex style="flex: 1" vertical>
            <n-button
              :disabled="show_add_redirect_id_btn"
              @click="add_redirect_id"
            >
              运行容器为 Flow 出口
            </n-button>
            <n-dynamic-input
              v-model:value="formModel.labels"
              preset="pair"
              separator=":"
              key-placeholder="key"
              value-placeholder="value"
            />
          </n-flex>
        </n-form-item>
      </n-form>
      <template #footer>
        <n-flex justify="end">
          <n-button
            :loading="save_loading"
            round
            type="primary"
            @click="save_config"
          >
            创建容器
          </n-button>
        </n-flex>
      </template>
    </n-card>
  </n-modal>
</template>
