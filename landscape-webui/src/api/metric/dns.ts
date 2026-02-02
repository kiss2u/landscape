import axiosService from "@/api";

export interface DnsMetric {
  flow_id: number;
  domain: string;
  query_type: string;
  response_code: string;
  status: string;
  report_time: number;
  duration_ms: number;
  src_ip: string;
  answers: string[];
}

export interface DnsHistoryResponse {
  items: DnsMetric[];
  total: number;
}

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
