import { createRouter, createWebHashHistory, RouteRecordRaw } from "vue-router";

import Landscape from "@/views/Landscape.vue";
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
];

const router = createRouter({ history: createWebHashHistory(), routes });

export default router;
