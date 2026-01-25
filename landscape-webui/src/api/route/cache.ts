import axiosService from "@/api";

export async function reset_cache(): Promise<void> {
  let data = await axiosService.post(`services/route/reset_cache`);
}
