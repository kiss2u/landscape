import { createRouter, createWebHashHistory, RouteRecordRaw } from "vue-router";

import Landscape from "@/views/Landscape.vue";
import LandscapeV2 from "@/views/LandscapeV2.vue";
import MainLayout from "@/views/MainLayout.vue";
import Flow from "@/views/Flow.vue";
import Docker from "@/views/Docker.vue";
import Topology from "@/views/Topology.vue";
import Firewall from "@/views/Firewall.vue";
import Metric from "@/views/Metric.vue";

import Login from "@/views/Login.vue";

const inner_zone: Array<RouteRecordRaw> = [
  {
    path: "/",
    name: "",
    component: Landscape,
  },
  {
    path: "/flow",
    name: "flow",
    component: Flow,
  },
  {
    path: "/topology",
    name: "topology",
    component: Topology,
  },
  {
    path: "/docker",
    name: "docker",
    component: Docker,
  },
  {
    path: "/firewall",
    name: "firewall",
    component: Firewall,
  },
  {
    path: "/metric",
    name: "metric",
    component: Metric,
  },
];

const routes: Array<RouteRecordRaw> = [
  {
    path: "/",
    name: "MainLayout",
    component: MainLayout,
    children: [...inner_zone],
  },
  {
    path: "/login",
    name: "Login",
    component: Login,
  },
  {
    path: "/test",
    name: "LandscapeV2",
    component: LandscapeV2,
  },
];

const router = createRouter({ history: createWebHashHistory(), routes });

export default router;
