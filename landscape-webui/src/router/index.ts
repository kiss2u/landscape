import { createRouter, createWebHashHistory, RouteRecordRaw } from "vue-router";

import Landscape from "@/views/Landscape.vue";
import LandscapeV2 from "@/views/LandscapeV2.vue";
import MainLayout from "@/views/MainLayout.vue";

import Login from "@/views/Login.vue";

const inner_zone: Array<RouteRecordRaw> = [
  {
    path: "/",
    name: "Landscape",
    component: Landscape,
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
