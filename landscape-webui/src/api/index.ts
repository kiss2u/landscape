import axios from "axios";
import router from "@/router";
import { LANDSCAPE_TOKEN_KEY } from "@/lib/common";

const base_url = import.meta.env.VITE_AXIOS_BASE_URL;

const axiosService = axios.create({
  baseURL: `${base_url}/src`,
  timeout: 30000,
});

axiosService.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem(LANDSCAPE_TOKEN_KEY);
    if (token) {
      config.headers["Authorization"] = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  },
);

axiosService.interceptors.response.use(
  (response) => response.data,
  (error) => {
    if (error.response != undefined && error.response.status != undefined) {
      const code = error.response.status;
      const msg = error.response.data.message;
      if (code === 401) {
        localStorage.removeItem(LANDSCAPE_TOKEN_KEY);

        router.push({
          path: "/login",
          state: { redirect: router.currentRoute.value.fullPath },
        });
      }

      if (msg && window.$message) {
        window.$message.error(msg);
      }
      return Promise.reject(error.response.data);
    }
    return Promise.reject(error);
  },
);

export default axiosService;
