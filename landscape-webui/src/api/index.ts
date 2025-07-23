import axios from "axios";
import router from "@/router";
import { LANDSCAPE_TOKEN_KEY } from "@/lib/common";

const base_url = import.meta.env.VITE_AXIOS_BASE_URL;

const axiosService = axios.create({
  baseURL: `${base_url}/src`,
  timeout: 10000,
});

axiosService.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem(LANDSCAPE_TOKEN_KEY);
    if (token) {
      // 如果存在 token，则将其添加到请求头
      config.headers["Authorization"] = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

axiosService.interceptors.response.use(
  (response) => response.data,
  (error) => {
    if (error.response != undefined && error.response.status != undefined) {
      const code = error.response.status;
      const msg = error.response.data.message;
      if (code === 401) {
        // 清除本地存储中的认证信息
        localStorage.removeItem("token");

        // 重定向到登录页面
        router.push({
          path: "/login",
          // query: { redirect: router.currentRoute }, // 登录成功后重定向回原页面
        });
      }

      if (msg && window.$message) {
        window.$message.error(msg);
      }
      return Promise.reject(error.response.data);
    }
    return Promise.reject(error);
  }
);

export default axiosService;
