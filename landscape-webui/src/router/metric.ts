import { RouteRecordRaw } from "vue-router";
import DNSMetric from "@/views/metric/DNSMetric.vue";
import LiveMetric from "@/views/metric/conn/LiveMetric.vue";
import HistoryMetric from "@/views/metric/conn/HistoryMetric.vue";
import SrcIpMetric from "@/views/metric/conn/SrcIpMetric.vue";
import DstIpMetric from "@/views/metric/conn/DstIpMetric.vue";
import HistorySrcIpMetric from "@/views/metric/conn/HistorySrcIpMetric.vue";
import HistoryDstIpMetric from "@/views/metric/conn/HistoryDstIpMetric.vue";

const metric_route: Array<RouteRecordRaw> = [
  {
    path: "/metric/conn/live",
    name: "routes.connect-live",
    component: LiveMetric,
  },
  {
    path: "/metric/conn/history",
    name: "routes.connect-history",
    component: HistoryMetric,
  },
  {
    path: "/metric/conn/src",
    name: "routes.connect-src",
    component: SrcIpMetric,
  },
  {
    path: "/metric/conn/dst",
    name: "routes.connect-dst",
    component: DstIpMetric,
  },
  {
    path: "/metric/conn/history-src",
    name: "routes.connect-history-src",
    component: HistorySrcIpMetric,
  },
  {
    path: "/metric/conn/history-dst",
    name: "routes.connect-history-dst",
    component: HistoryDstIpMetric,
  },
  {
    path: "/metric/dns",
    name: "routes.dns-metric",
    component: DNSMetric,
  },
];

export default metric_route;
