import { RouteRecordRaw } from "vue-router";
import ConnectMetric from "@/views/metric/ConnectMetric.vue";
import DNSMetric from "@/views/metric/DNSMetric.vue";
import LiveMetric from "@/views/metric/conn/LiveMetric.vue";
import HistoryMetric from "@/views/metric/conn/HistoryMetric.vue";
import SrcIpMetric from "@/views/metric/conn/SrcIpMetric.vue";
import DstIpMetric from "@/views/metric/conn/DstIpMetric.vue";

const metric_route: Array<RouteRecordRaw> = [
  {
    path: "/metric/conn",
    name: "connect-metric",
    component: ConnectMetric,
    redirect: "/metric/conn/live",
    children: [
      {
        path: "live",
        name: "connect-live",
        component: LiveMetric,
      },
      {
        path: "history",
        name: "connect-history",
        component: HistoryMetric,
      },
      {
        path: "src",
        name: "connect-src",
        component: SrcIpMetric,
      },
      {
        path: "dst",
        name: "connect-dst",
        component: DstIpMetric,
      },
    ],
  },
  {
    path: "/metric/dns",
    name: "dns-metric",
    component: DNSMetric,
  },
];

export default metric_route;
