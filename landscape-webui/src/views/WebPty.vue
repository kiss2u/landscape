<script setup lang="ts">
import { SerializeAddon } from "@xterm/addon-serialize";
import { Terminal } from "@xterm/xterm";
import { onBeforeUnmount, onMounted, ref } from "vue";
import { LANDSCAPE_TOKEN_KEY } from "@/lib/common";
import { LandscapePtyConfig, PtyOutMessage } from "landscape-types/common/pty";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";

const terminal = ref(null as unknown as HTMLDivElement);
const term_obj = ref<Terminal | undefined>();
const sock = ref<WebSocket | undefined>();
const is_connected = ref(false);
const query = ref<LandscapePtyConfig>({
  shell: "bash",
  rows: 0,
  cols: 0,
  pixel_width: 0,
  pixel_height: 0,
});

const resizeTimeout = ref<number>();

function objToQuery(obj: any) {
  const token = localStorage.getItem(LANDSCAPE_TOKEN_KEY) ?? "";
  const query = Object.entries(obj)
    .map(
      ([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(String(v))}`
    )
    .join("&");

  // 如果 obj 可能为空，避免多余的 &
  return query
    ? `${query}&token=${encodeURIComponent(token)}`
    : `token=${encodeURIComponent(token)}`;
}
function reconnect_socket() {
  disconnect_socket();
  connect_socket();
}

function disconnect_socket() {
  sock.value?.close();
  sock.value?.send(
    JSON.stringify({
      t: "exit",
    })
  );
}

function connect_socket() {
  const current_sock = new WebSocket(
    `wss://${window.location.hostname}:${
      window.location.port
    }/api/sock/pty/create_session?${objToQuery(query.value)}`
  );

  current_sock.addEventListener("message", (event) => {
    const msg = JSON.parse(event.data) as PtyOutMessage;
    if (msg.t === "data") {
      const text = String.fromCharCode(...msg.data);
      term_obj.value?.write(text);
    } else if (msg.t === "exit") {
      console.log("PTY exited:", msg.msg);
    }
  });

  current_sock.addEventListener("open", function () {
    is_connected.value = true;
  });

  current_sock.addEventListener("close", function () {
    is_connected.value = false;
  });
  current_sock.onerror = () => {
    is_connected.value = false;
  };

  sock.value = current_sock;
}

function create_terminal() {
  const term = new Terminal({
    cols: 129, // 9px
    rows: 33, // 17px
    // rendererType: 'dom'
  });

  const serializeAddon = new SerializeAddon();
  const fit = new FitAddon();
  term.onResize((a: { cols: number; rows: number }) => {
    // console.log(`on resize: ${JSON.stringify(a)}`);
    sock.value?.send(
      JSON.stringify({
        t: "size",
        size: { ...a, pixel_width: 0, pixel_height: 0 },
      })
    );
  });

  term.onData((data) => {
    if (sock.value) {
      //   console.log(data);
      sock.value?.send(
        JSON.stringify({
          t: "data",
          data: Array.from(new TextEncoder().encode(data)),
        })
      );
    }
  });

  term.loadAddon(serializeAddon);
  term.loadAddon(fit);

  if (terminal.value != null) {
    term.open(terminal.value as HTMLDivElement);
  }

  let size = fit.proposeDimensions();
  if (size) {
    query.value.rows = size.rows;
    query.value.cols = size.cols;
  }

  let lastDims: { rows: number; cols: number } | null = null;

  const resizeObserver = new ResizeObserver((e) => {
    if (resizeTimeout.value) clearTimeout(resizeTimeout.value);
    resizeTimeout.value = window.setTimeout(() => {
      const dims = fit.proposeDimensions();
      if (!dims) return;

      if (
        !lastDims ||
        lastDims.rows !== dims.rows ||
        lastDims.cols !== dims.cols
      ) {
        lastDims = dims;
        fit.fit();
        // console.log(`Terminal resized to ${dims.cols} x ${dims.rows}`);
      }
    }, 100); // 100ms 防抖
  });

  if (terminal.value != null && terminal.value.parentElement != null) {
    resizeObserver.observe(terminal.value.parentElement as Element);
  }
  fit.fit();
  lastDims = fit.proposeDimensions() || null;
  term_obj.value = term;
}

onMounted(() => {
  create_terminal();
});
onBeforeUnmount(() => {
  disconnect_socket();
});
</script>

<template>
  <n-flex vertical style="flex: 1">
    <n-flex>
      <n-input
        style="width: 260px"
        :disabled="is_connected"
        clearable
        v-model:value="query.shell"
        placeholder="使用的 shell 程序"
      ></n-input>
      <n-button
        ghost
        :disabled="is_connected"
        type="success"
        @click="connect_socket"
      >
        连接
      </n-button>

      <n-popconfirm
        @positive-click="disconnect_socket"
        :show-icon="false"
        :positive-button-props="{ type: 'error', ghost: true }"
      >
        <template #trigger>
          <n-button ghost :disabled="!is_connected" type="error">
            断开
          </n-button>
        </template>
        确定断开连接吗
      </n-popconfirm>

      <n-popconfirm
        @positive-click="reconnect_socket"
        :show-icon="false"
        :positive-button-props="{ type: 'info', ghost: true }"
      >
        <template #trigger>
          <n-button ghost :disabled="!is_connected" type="info">
            重连
          </n-button>
        </template>
        确定重新连接吗
      </n-popconfirm>
    </n-flex>
    <n-flex class="parent">
      <div
        style="flex: 1"
        class="terminal-wrapper"
        ref="terminal"
        id="terminal"
      ></div>
    </n-flex>
  </n-flex>
</template>

<style scoped>
.parent {
  position: relative;
  flex: 1;
  overflow: hidden;
}

.terminal-wrapper {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 20px;
}
</style>
