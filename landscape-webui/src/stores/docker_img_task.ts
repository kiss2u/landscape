import { PullManagerInfo } from "@/rust_bindings/common/docker";
import { defineStore } from "pinia";
import { ref, computed } from "vue";

export const useDockerImgTask = defineStore("docker-img_task", () => {
  const socket = ref<WebSocket | undefined>(undefined);

  const tasks = ref<any[]>([]);

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
      if (tasks.value.length >= 60) {
        tasks.value.shift();
      }
      tasks.value.push(JSON.parse(event.data));
    });
  }

  function DISCONNECT() {
    if (socket.value) {
      socket.value.close();
    }
  }

  return {
    tasks,
    CONNECT,
    DISCONNECT,
  };
});

export default useDockerImgTask;
