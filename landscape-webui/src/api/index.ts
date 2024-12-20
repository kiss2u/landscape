import axios from "axios";

const instance = axios.create({
  baseURL: import.meta.env.VITE_AXIOS_BASE_URL,
  timeout: 1000,
});

export default {
  api: instance,
};
