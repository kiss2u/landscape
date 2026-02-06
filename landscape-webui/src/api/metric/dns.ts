import axiosService from "@/api";
import type {
  DnsMetric,
  DnsHistoryResponse,
  DnsStatEntry,
  DnsSummaryResponse,
  DnsLightweightSummaryResponse,
} from "landscape-types/common/metric/dns";

export type {
  DnsMetric,
  DnsHistoryResponse,
  DnsStatEntry,
  DnsSummaryResponse,
  DnsLightweightSummaryResponse,
};

export async function get_dns_history(
  params: {
    start_time?: number;
    end_time?: number;
    limit?: number;
    offset?: number;
    sort_key?: string;
    sort_order?: string;
    domain?: string;
    src_ip?: string;
    flow_id?: number;
  } = {},
): Promise<DnsHistoryResponse> {
  let data = await axiosService.get("metric/dns/history", {
    params,
  });
  return data.data;
}

export async function get_dns_summary(params: {
  start_time: number;
  end_time: number;
  flow_id?: number;
}): Promise<DnsSummaryResponse> {
  let data = await axiosService.get("metric/dns/summary", {
    params,
  });
  return data.data;
}

export async function get_dns_lightweight_summary(params: {
  start_time: number;
  end_time: number;
}): Promise<DnsLightweightSummaryResponse> {
  let data = await axiosService.get("metric/dns/summary/lightweight", {
    params,
  });
  return data.data;
}
