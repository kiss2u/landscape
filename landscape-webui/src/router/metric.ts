import { RouteRecordRaw } from "vue-router";
import ConnectMetric from "@/views/metric/ConnectMetric.vue";
import DNSMetric from "@/views/metric/DNSMetric.vue";

const metric_route: Array<RouteRecordRaw> = [
  {
    path: "/metric/conn",
    name: "connect-metric",
    component: ConnectMetric,
  },
  {
    path: "/metric/dns",
    name: "dns-metric",
    component: DNSMetric,
  },
];

export default metric_route;
