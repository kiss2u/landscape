import { get_capabilities } from "@/api/sys";
import { defineStore } from "pinia";
import { ref } from "vue";

export const useCapabilityStore = defineStore("capability", () => {
  const capabilities = ref<Set<string>>(new Set());
  const loaded = ref(false);
  let pending: Promise<void> | null = null;

  async function LOAD() {
    if (loaded.value) return;
    if (pending) return pending;
    pending = (async () => {
      try {
        const list = await get_capabilities();
        capabilities.value = new Set(list as string[]);
        loaded.value = true;
      } catch {
        // The backend may be an older build without the capability endpoint,
        // or the request failed. Keep `loaded=false` so HAS() fails open.
      } finally {
        pending = null;
      }
    })();
    return pending;
  }

  /**
   * Whether the given capability is available. Before the list has been loaded
   * (or when loading failed) this returns `true` so the UI stays fully usable.
   */
  function HAS(capability: string): boolean {
    if (!loaded.value) return true;
    return capabilities.value.has(capability);
  }

  function RESET() {
    capabilities.value = new Set();
    loaded.value = false;
    pending = null;
  }

  return {
    capabilities,
    loaded,
    LOAD,
    HAS,
    RESET,
  };
});
