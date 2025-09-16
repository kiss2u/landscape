import { RouteRecordRaw } from "vue-router";

import WebPty from "@/views/WebPty.vue";

const web_pty_route: Array<RouteRecordRaw> = [
  {
    path: "/web-pty",
    name: "web-pty",
    component: WebPty,
  },
];

export default web_pty_route;
