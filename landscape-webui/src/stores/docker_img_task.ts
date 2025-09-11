import { get_current_tasks } from "@/api/docker";
import { ImgPullEvent, PullImgTask } from "@/rust_bindings/common/docker";
import { defineStore } from "pinia";
import { ref, computed } from "vue";

export const useDockerImgTask = defineStore("docker-img_task", () => {
  const socket = ref<WebSocket | undefined>(undefined);

  const tasks = ref<PullImgTask[]>([]);

  function CONNECT() {
    if (socket.value && socket.value.readyState === WebSocket.OPEN) {
      socket.value.send(JSON.stringify({ type: "ping" }));
      return;
    }

    socket.value = new WebSocket(
      `wss://${window.location.hostname}:${window.location.port}/api/sock/docker/listen_docker_task`
    );
    socket.value.addEventListener("open", function (event) {
      socket.value?.send("Hello Server!");
    });

    socket.value.addEventListener("message", function (event) {
      console.log("Message from server ", event.data);
      let data = JSON.parse(event.data) as ImgPullEvent;
      for (const task of tasks.value) {
        if (task.id == data.task_id) {
          task.layer_current_info[data.id] = data;
        }
      }
    });
  }

  async function INIT() {
    tasks.value = await get_current_tasks();
  }

  function DISCONNECT() {
    if (socket.value) {
      socket.value.close();
    }
  }

  return {
    tasks,
    INIT,
    CONNECT,
    DISCONNECT,
  };
});

export default useDockerImgTask;
