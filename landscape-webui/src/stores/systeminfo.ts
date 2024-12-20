import { get_cpu, get_mem } from "@/api/sys";
import { defineStore } from "pinia";
import { ref } from "vue";

export const useSysInfo = defineStore("sysinfo", () => {
  const cpus = ref<any>();
  const mem = ref<any>({});

  async function UPDATE_INFO() {
    mem.value = await get_mem();
    cpus.value = await get_cpu();
  }

  return {
    cpus,
    mem,
    UPDATE_INFO,
  };
});
