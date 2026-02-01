<script setup lang="ts">
import { NDrawer, NDrawerContent, NList, NListItem } from "naive-ui";
import { ref } from "vue";

const show_switch = defineModel<boolean>();

const socket = ref<WebSocket | undefined>(undefined);
function enter() {
  socket.value = new WebSocket(
    `ws://${window.location.hostname}:${window.location.port}/ws/sock/dump/br0-test`,
  );
  //     socket.value.addEventListener('open', function (event) {
  // });
  socket.value.addEventListener("open", function (event) {
    socket.value?.send("Hello Server!");
  });

  socket.value.addEventListener("message", function (event) {
    console.log("Message from server ", event.data);
    if (items.value.length >= 10) {
      items.value.shift();
    }
    items.value.push(event.data);
  });
}

function exit() {
  if (socket.value !== undefined) {
    socket.value.close();
  }
  items.value.splice(0, items.value.length);
}
const items = ref<any[]>([]);
</script>
<template>
  <!-- {{ show_switch }} -->
  <n-drawer
    v-model:show="show_switch"
    :height="600"
    placement="bottom"
    @after-enter="enter"
    @after-leave="exit"
  >
    <n-drawer-content title="">
      <n-list
        ref="virtualListInst"
        style="max-height: 240px"
        :item-size="42"
        :items="items"
      >
        <n-list-item v-for="(item, index) of items">
          {{ index }} {{ item }}
        </n-list-item>
      </n-list>
    </n-drawer-content>
  </n-drawer>
</template>
