import axiosService from "@/api";

export async function get_init_config(): Promise<void> {
  try {
    const response = await axiosService.get(`sys_service/config/export`);
    const jsonStr = response.data;

    let filename = "landscape_init.toml";

    const blob = new Blob([jsonStr], { type: "application/octet-stream" });
    const url = window.URL.createObjectURL(blob);

    // 创建 a 标签模拟点击
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();

    // 释放 URL
    window.URL.revokeObjectURL(url);
  } catch (error) {
    console.error("下载配置失败", error);
  }
}
