import { createRouter, createWebHashHistory, RouteRecordRaw } from "vue-router";

import Landscape from "@/views/Landscape.vue";
import LandscapeV2 from "@/views/LandscapeV2.vue";
import Login from "@/views/Login.vue";

const routes: Array<RouteRecordRaw> = [
  {
    path: "/",
    name: "Landscape",
    component: Landscape,
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
