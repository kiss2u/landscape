<script setup lang="ts">
import type { MenuOption } from "naive-ui";
import type { Component } from "vue";

import { NIcon } from "naive-ui";
import { h, ref } from "vue";
import { Network4, CicsSystemGroup, ModelBuilder } from "@vicons/carbon";

import { Wall } from "@vicons/tabler";
import { ImportExportRound } from "@vicons/material";
import { Docker } from "@vicons/fa";

import CopyRight from "@/components/CopyRight.vue";

const activeKey = ref<string | null>(null);
const collapsed = ref(true);

function renderIcon(icon: Component) {
  return () => h(NIcon, null, { default: () => h(icon) });
}

const menuOptions: MenuOption[] = [
  {
    label: "系统基本信息",
    key: "info",
    icon: renderIcon(CicsSystemGroup),
  },
  {
    label: "网络拓扑",
    key: "topology",
    icon: renderIcon(Network4),
  },
  {
    label: "NAT",
    key: "nat",
    icon: renderIcon(ImportExportRound),
  },
  {
    label: "防火墙",
    key: "firewall",
    icon: renderIcon(Wall),
  },
  //   {
  //     label: "网络管理",
  //     key: "net",
  //     icon: renderIcon(NetworkPublic),
  //     children: [
  //       {
  //         label: "NAT",
  //         key: "nat",
  //       },
  //       {
  //         label: "防火墙",
  //         key: "firewall",
  //       },
  //     ],
  //   },
  {
    label: "分流设置",
    key: "flow",
    icon: renderIcon(ModelBuilder),
  },
  {
    label: "Docker",
    key: "docker",
    icon: renderIcon(Docker),
  },
];
</script>
<template>
  <n-layout-sider
    position="relative"
    :native-scrollbar="false"
    bordered
    collapse-mode="width"
    :collapsed-width="64"
    :width="240"
    :collapsed="collapsed"
    show-trigger="bar"
    @collapse="collapsed = true"
    @expand="collapsed = false"
  >
    <n-layout position="absolute">
      <n-layout-header
        v-if="!collapsed"
        style="height: 30px; display: flex"
        bordered
      >
        <n-flex justify="center" style="flex: 1" align="center">
          Landscape
        </n-flex>
      </n-layout-header>
      <n-layout position="absolute" style="top: 30px; bottom: 64px">
        <n-menu
          v-model:value="activeKey"
          :collapsed="collapsed"
          :collapsed-width="64"
          :collapsed-icon-size="22"
          :options="menuOptions"
        />
      </n-layout>
      <n-layout-footer
        bordered
        position="absolute"
        content-style="dispaly: flex; height: 30px"
      >
        <n-flex
          style="flex: 1; height: 30px"
          :justify="collapsed ? 'center' : 'start'"
          align="center"
        >
          <CopyRight :icon="true"></CopyRight>
        </n-flex>
      </n-layout-footer>
    </n-layout>
  </n-layout-sider>
</template>
