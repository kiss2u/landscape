import { RouteRecordRaw } from "vue-router";
import DNSMetric from "@/views/metric/DNSMetric.vue";
import LiveMetric from "@/views/metric/conn/LiveMetric.vue";
import HistoryMetric from "@/views/metric/conn/HistoryMetric.vue";
import SrcIpMetric from "@/views/metric/conn/SrcIpMetric.vue";
import DstIpMetric from "@/views/metric/conn/DstIpMetric.vue";

const metric_route: Array<RouteRecordRaw> = [
  {
    path: "/metric/conn/live",
    name: "connect-live",
    component: LiveMetric,
  },
  {
    path: "/metric/conn/history",
    name: "connect-history",
    component: HistoryMetric,
  },
  {
    path: "/metric/conn/src",
    name: "connect-src",
    component: SrcIpMetric,
  },
  {
    path: "/metric/conn/dst",
    name: "connect-dst",
    component: DstIpMetric,
  },
  {
    path: "/metric/dns",
    name: "dns-metric",
    component: DNSMetric,
  },
];

export default metric_route;
