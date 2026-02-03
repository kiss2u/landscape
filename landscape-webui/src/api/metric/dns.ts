import axiosService from "@/api";
import type {
  DnsMetric,
  DnsHistoryResponse,
  DnsStatEntry,
  DnsSummaryResponse,
} from "landscape-types/common/metric/dns";

export type { DnsMetric, DnsHistoryResponse, DnsStatEntry, DnsSummaryResponse };

export async function get_dns_history(params?: {
  start_time?: number;
  end_time?: number;
  limit?: number;
  offset?: number;
  sort_key?: string;
  sort_order?: string;
  domain?: string;
  src_ip?: string;
}): Promise<DnsHistoryResponse> {
  let data = await axiosService.get("metric/dns/history", {
    params,
  });
  return data.data;
}

export async function get_dns_summary(params?: {
  start_time?: number;
  end_time?: number;
}): Promise<DnsSummaryResponse> {
  let data = await axiosService.get("metric/dns/summary", {
    params,
  });
  return data.data;
}
