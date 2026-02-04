import metric from "./metric/dns";
import sysinfo from "./sysinfo";
import config from "./config";
import error from "./error";

export default {
  docker_divider: "Docker Containers",
  topology_divider: "Network topology",
  metric,
  sysinfo,
  config,
  error,
  routes: {
    dashboard: "Dashboard",
    "dns-redirect": "DNS Redirect",
    "dns-upstream": "Upstream DNS",
    nat: "Static NAT",
    flow: "Traffic Flow",
    topology: "Network Topology",
    docker: "Docker Management",
    firewall: "Firewall",
    "geo-domain": "Geo Domain",
    "geo-ip": "Geo IP",
    config: "System Config",
    "metric-group": "Metrics",
    "connect-live": "Active Connections",
    "connect-history": "History Query",
    "connect-src": "Src IP Stats",
    "connect-dst": "Dst IP Stats",
    "dns-metric": "DNS Metrics",
    "ipv6-pd": "IPv6 PD",
    "dhcp-v4": "DHCPv4 Service",
    "ipv6-ra": "IPv6 RA",
  },
};
